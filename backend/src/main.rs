#[macro_use]
extern crate rocket;

use analyzer::{AnalysisStatus, Analyzer, Metadata};
use chrono::{DateTime, Utc};
use entities::{analysis, sample};
use md5::Digest;
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::{Request, State};
use sea_orm::*;
use sha2::Sha512;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use anyhow::anyhow;
use clap::builder::styling;
use clap::{CommandFactory, FromArgMatches, Parser};
use rocket::form::Form;
use rocket::fs::{NamedFile, TempFile};
use rocket::serde::json::Json;
use rust_embed::Embed;
use serde::Serialize;

mod analyzer;
mod config;
mod db;
mod entities;

use config::Config;

use entities::prelude::Analysis as DbAnalysis;
use entities::prelude::Sample as DbSample;

const API_MOUNTPOINT: &str = "/api";

macro_rules! api_error {
    ($msg: expr) => {
        ApiError::msg(format!("{}:{} {}", file!(), line!(), $msg))
    };
}

#[derive(ToSchema, Serialize)]
struct ApiResponse<D: Serialize> {
    error: Option<String>,
    data: Option<D>,
}

#[derive(ToSchema, Serialize)]
enum ApiData<D: Serialize> {
    Some(D),
    None,
}

impl<D> From<Option<D>> for ApiData<D>
where
    D: Serialize,
{
    fn from(value: Option<D>) -> Self {
        match value {
            Some(d) => ApiData::Some(d),
            None => ApiData::None,
        }
    }
}

impl<D> From<ApiData<D>> for Option<D>
where
    D: Serialize,
{
    fn from(value: ApiData<D>) -> Self {
        match value {
            ApiData::Some(d) => Some(d),
            ApiData::None => None,
        }
    }
}

impl<'r, D> Responder<'r, 'static> for ApiData<D>
where
    D: Serialize,
{
    fn respond_to(self, r: &'r Request<'_>) -> rocket::response::Result<'static> {
        let json = Json(ApiResponse {
            data: Option::<D>::from(self),
            error: None,
        });

        json.respond_to(r)
    }
}

type ApiResult<T> = Result<ApiData<T>, ApiError>;

#[derive(Debug, Error)]
enum ApiError {
    #[error("{0}")]
    Msg(String),
}

impl ApiError {
    fn msg<S: AsRef<str>>(s: S) -> Self {
        ApiError::Msg(s.as_ref().to_string())
    }
}

// Implement the ResponseError trait for ApiError
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, r: &'r Request<'_>) -> rocket::response::Result<'static> {
        let json = Json(ApiResponse::<()> {
            error: Some(self.to_string()),
            data: None,
        });

        json.respond_to(r)
    }
}

struct ClientIp(IpAddr);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let forwarded_for = request.headers().get_one("X-Forwarded-For");
        if let Some(forwarded_for) = forwarded_for {
            // The X-Forwarded-For header can contain multiple IPs, separated by commas.
            // The first IP is the original client IP.
            if let Some(ip) = forwarded_for.split(',').next() {
                if let Ok(ip) = ip.trim().parse::<IpAddr>() {
                    return Outcome::Success(ClientIp(ip));
                }
            }
        }

        // Fallback to the direct client IP if X-Forwarded-For is not present or invalid.
        if let Some(ip) = request.client_ip() {
            return Outcome::Success(ClientIp(ip));
        }

        Outcome::Error((Status::BadRequest, ()))
    }
}

#[derive(FromForm, ToSchema)]
struct Upload<'r> {
    #[schema(value_type = String, format = Binary)]
    file: TempFile<'r>,
    sandbox: Option<String>,
}

async fn sha512<R: tokio::io::AsyncBufRead + Unpin>(mut reader: R) -> tokio::io::Result<String> {
    let mut sha512 = Sha512::new();

    let mut buf = [0; 4096];

    let mut n = reader.read(&mut buf[..]).await?;
    while n > 0 {
        sha512.update(&buf[..n]);
        n = reader.read(&mut buf[..]).await?;
    }

    Ok(hex::encode(sha512.finalize()))
}

#[utoipa::path(
    context_path = API_MOUNTPOINT,
    request_body(
        content = Upload,
        description = "Uploaded file and optional sandbox name",
        content_type = "multipart/form-data"),
        responses(
            (status = 200, description = "New analysis UUID", body = ApiResponse<Uuid>),
        ),
        tag = "analysis",
        description = "Analyzes an uploaded file and returns a new analysis UUID."
    )]
#[post("/analyze", data = "<upload>")]
async fn analyze(
    mut upload: Form<Upload<'_>>,
    config: &State<Config>,
    analyzer: &State<Arc<Mutex<Analyzer>>>,
    client_ip: ClientIp,
) -> ApiResult<Uuid> {
    // get submission name
    let sub_name = upload
        .file
        .raw_name()
        .map(|rn| rn.dangerous_unsafe_unsanitized_raw())
        .map(|s| s.to_string());

    let tmp_reader = upload
        .file
        .open()
        .await
        .inspect_err(|e| error!("failed to read temporary file: {e}"))
        .map_err(|_| api_error!("failed to read temporary file"))?;

    let hash = sha512(tmp_reader)
        .await
        .inspect_err(|e| error!("failed to compute hash: {e}"))
        .map_err(|_| api_error!("failed to compute hash"))?;

    let s = DbSample::find()
        .filter(sample::Column::Sha512.eq(hash))
        .one(&analyzer.lock().await.db)
        .await
        .inspect_err(|e| error!("failed  to make sha512 query in database: {e}"))
        .map_err(|_| api_error!("failed to make sha512 query in database"))?;

    let sample_in_db = s.is_some();

    let sample_uuid = match s {
        Some(s) => s.uuid,
        None => Uuid::new_v4(),
    };

    let mut analysis = analyzer::Analysis::new(
        config.inner(),
        upload.sandbox.take(),
        sample_uuid,
        sub_name,
        client_ip.0,
    );

    upload
        .file
        .copy_to(&analysis.sample_path())
        .await
        .inspect_err(|e| error!("failed to copy file: {e}"))
        .map_err(|_| api_error!("failed to copy file"))?;

    // FIXME: delete sample if we fail at returning
    let sample_uuid = analysis.sample_uuid;
    let metadata = analysis
        .async_metadata()
        .await
        .inspect_err(|e| error!("failed to get submission metadata: {e}"))
        .map_err(|_| api_error!("failed to get submission metadata"))?;

    // if sample already exists we do not insert it again otherwise
    // we are going to hit a db uniqueness constraint
    if !sample_in_db {
        let s = sample::ActiveModel {
            uuid: ActiveValue::Set(sample_uuid),
            md5: ActiveValue::Set(metadata.md5.clone()),
            sha1: ActiveValue::Set(metadata.sha1.clone()),
            sha256: ActiveValue::Set(metadata.sha256.clone()),
            sha512: ActiveValue::Set(metadata.sha512.clone()),
        };

        s.insert(&analyzer.lock().await.db)
            .await
            .inspect_err(|e| error!("failed to instert sample into database: {e}"))
            .map_err(|_| api_error!("failed to insert sample into database"))?;
    }

    let uuid = analysis.analysis_uuid;

    // we need to schedule analysis here
    analyzer
        .lock()
        .await
        .queue_new(&analysis)
        .await
        .inspect_err(|e| error!("failed to queue analysis: {e}"))
        .map_err(|_| api_error!("failed to queue analysis"))?;

    Ok(ApiData::Some(uuid))
}

#[utoipa::path(
    context_path = API_MOUNTPOINT,
    params(
        ("analysis_uuid" = String, Path, description = "The UUID of the analysis to re-run"),
        ("sandbox" = Option<String>, Query, description = "Optional sandbox name for the analysis")
    ),
    responses(
        (status = 200, description = "New analysis UUID", body = ApiResponse<Uuid>),
    ),
    tag = "analysis",
    description = "Re-runs an analysis using the provided UUID and returns a new analysis UUID."
)]
#[post("/analyze/again/<analysis_uuid>?<sandbox>")]
async fn analyze_again(
    analysis_uuid: &str,
    sandbox: Option<&str>,
    config: &State<Config>,
    analyzer: &State<Arc<Mutex<Analyzer>>>,
    db: &State<DatabaseConnection>,
    client_ip: ClientIp,
) -> ApiResult<Uuid> {
    let uuid = Uuid::from_str(analysis_uuid).map_err(|_| api_error!("cannot parse uuid"))?;

    let search_res = DbAnalysis::find_by_id(uuid)
        .one(db.inner())
        .await
        .inspect_err(|e| error!("failed to search sample by analysis uuid: {e}"))
        .map_err(|_| api_error!("failed to search sample by analysis uuid"))?;

    let analysis = match search_res {
        Some(analysis) => analysis,
        _ => return Ok(ApiData::None),
    };

    let status = AnalysisStatus::from_str(&analysis.status)
        .inspect_err(|_| error!("failed to convert analysis status from string"))
        .map_err(|_| api_error!("failed to convert analysis status from string"))?;

    let analysis = match status {
        // if analysis failed or if it couldn't be queued, we can re-use analysis UUID
        AnalysisStatus::unqueued | AnalysisStatus::failed => {
            let a = analyzer::Analysis::from_model_with_config(analysis, config.inner())
                .inspect_err(|e| error!("failed to convert analysis from db: {e}"))
                .map_err(|_| api_error!("failed to convert analysis from db"))?;

            // queue an existing analysis has a different DB logic
            analyzer
                .lock()
                .await
                .queue_existing(&a)
                .await
                .inspect_err(|e| error!("failed to queue analysis: {e}"))
                .map_err(|_| api_error!("failed to queue analysis"))?;

            a
        }

        // we create a new analysis with its own UUID so that we do not overwrite old analysis
        AnalysisStatus::queued | AnalysisStatus::running | AnalysisStatus::terminated => {
            let a = analyzer::Analysis::new(
                config.inner(),
                sandbox.map(String::from).or(Some(analysis.sandbox_name)),
                analysis.sample_uuid,
                analysis.submission_name,
                client_ip.0,
            );

            // we queue a new analysis
            analyzer
                .lock()
                .await
                .queue_new(&a)
                .await
                .inspect_err(|e| error!("failed to queue analysis: {e}"))
                .map_err(|_| api_error!("failed to queue analysis"))?;

            a
        }
    };

    Ok(ApiData::Some(analysis.analysis_uuid))
}

#[derive(ToSchema, Serialize)]
pub struct Analysis {
    pub uuid: Uuid,
    pub date: DateTime<Utc>,
    pub submission_name: Option<String>,
    pub status: AnalysisStatus,
    pub sample: Option<sample::Model>,
}

impl From<(analysis::Model, Vec<sample::Model>)> for Analysis {
    fn from(value: (analysis::Model, Vec<sample::Model>)) -> Self {
        let mut samples = value.1;
        Self {
            uuid: value.0.uuid,
            date: value.0.date.and_utc(),
            submission_name: value.0.submission_name,
            status: AnalysisStatus::from_str(&value.0.status).unwrap(),
            sample: samples.pop(),
        }
    }
}

#[utoipa::path(
    context_path = API_MOUNTPOINT,
    params(
        ("limit" = Option<u64>, Query, description = "Limit the number of results, default is 25"),
        ("offset" = Option<u64>, Query, description = "Offset for pagination, default is 0"),
        ("hash" = Option<String>, Query, description = "Optional hash to filter results by sample MD5, SHA1, SHA256, or SHA512")
    ),
    responses(
        (status = 200, description = "List of analysis items", body = ApiResponse<Vec<Analysis>>),
    ),
    tag = "analyses",
    description = "Retrieves a list of analysis items with optional pagination and hash filtering."
)]
#[get("/analyses/search?<limit>&<offset>&<hash>&<status>")]
async fn analyses_search(
    limit: Option<u64>,
    offset: Option<u64>,
    hash: Option<&str>,
    status: Option<&str>,
    analyzer: &State<Arc<Mutex<Analyzer>>>,
) -> ApiResult<Vec<Analysis>> {
    let limit = limit.unwrap_or(25).clamp(0, 100);
    let offset = offset.unwrap_or_default();

    let mut query = DbAnalysis::find()
        .order_by(analysis::Column::Date, Order::Desc)
        .find_with_related(DbSample);

    if let Some(hash) = hash {
        query = query.filter(
            Condition::any()
                .add(sample::Column::Md5.eq(hash))
                .add(sample::Column::Sha1.eq(hash))
                .add(sample::Column::Sha256.eq(hash))
                .add(sample::Column::Sha512.eq(hash)),
        )
    }

    if let Some(status) = status {
        query = query.filter(analysis::Column::Status.eq(status))
    }

    let analyzer = analyzer.lock().await;

    let out: Vec<Analysis> = query
        .limit(limit)
        .offset(offset)
        .all(&analyzer.db)
        .await
        .map_err(|_| api_error!("failed to retrieve last analysis"))?
        .into_iter()
        .map(Analysis::from)
        .map(|mut a| {
            if analyzer.is_running(&a.uuid) {
                a.status = AnalysisStatus::running;
            }
            a
        })
        .collect();

    Ok(ApiData::Some(out))
}

#[utoipa::path(
    context_path = API_MOUNTPOINT,
    params(
        ("uuid" = String, Path, description = "The UUID of the analysis")
    ),
    responses(
        (status = 200, description = "Sandbox details for the analysis", body = ApiResponse<config::Sandbox>),
    ),
    tag = "analysis",
    description = "Retrieves the sandbox details for a specific analysis by UUID."
)]
#[get("/analysis/<uuid>/sandbox")]
async fn analysis_sandbox(uuid: &str, config: &State<Config>) -> ApiResult<config::Sandbox> {
    let uuid = Uuid::from_str(uuid).map_err(|_| api_error!("cannot parse uuid"))?;

    let data = tokio::fs::read_to_string(config.analysis_sandbox_json_file(&uuid))
        .await
        .inspect_err(|e| error!("failed to read sandbox json file: {e}"))
        .map_err(|_| api_error!("failed to read sandbox json file"))?;

    let sandbox: config::Sandbox = serde_json::from_str(&data)
        .inspect_err(|e| error!("failed to deserialize sandbox json file: {e}"))
        .map_err(|_| api_error!("failed to deserialize sandbox json file"))?;

    Ok(ApiData::Some(sandbox))
}

#[utoipa::path(
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "Todo item created successfully", body = ApiResponse<Option<AnalysisStatus>>),
        ),
        tag = "analysis",
        context_path = API_MOUNTPOINT
    )]
#[get("/analysis/<uuid>/status")]
async fn analysis_status(
    uuid: &str,
    analyzer: &State<Arc<Mutex<Analyzer>>>,
) -> ApiResult<AnalysisStatus> {
    let uuid = Uuid::from_str(uuid).map_err(|_| api_error!("failed to parse uuid"))?;

    let status = analyzer
        .lock()
        .await
        .analysis_status(&uuid)
        .await
        .inspect_err(|e| error!("failed to retrieve analysis status: {e}"))
        .map_err(|_| api_error!("failed to retrieve analysis status"))?;

    Ok(status.into())
}

#[utoipa::path(
        context_path = API_MOUNTPOINT,
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "Metadata for the analysis", body = ApiResponse<Metadata>),
        ),
        tag = "analysis",
        description = "Retrieves the metadata for a specific analysis by UUID."
    )]
#[get("/analysis/<uuid>/metadata")]
async fn analysis_metadata(uuid: &str, config: &State<Config>) -> ApiResult<Metadata> {
    let uuid = Uuid::from_str(uuid).map_err(|_| api_error!("failed to parse uuid"))?;
    let meta_path = config.sample_metadata_json_file(&uuid);
    let metadata = serde_json::from_str(
        &fs::read_to_string(&meta_path)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to open metadata file {}: {e}",
                    meta_path.to_string_lossy()
                )
            })
            .map_err(|_| api_error!("failed to open metadata file"))?,
    )
    .inspect_err(|e| error!("failed to deserialize metadata: {e}"))
    .map_err(|_| api_error!("failed to deserialize metadata file"))?;

    Ok(ApiData::Some(metadata))
}

#[utoipa::path(
        context_path = API_MOUNTPOINT,
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "PCAP file for the analysis", content_type="application/octet-stream"),
            (status = 404, description = "PCAP file not found")
        ),
        tag = "analysis",
        description = "Retrieves the PCAP file for a specific analysis by UUID."
    )]
#[get("/analysis/<uuid>/pcap")]
async fn analysis_pcap(uuid: String, config: &State<Config>) -> Option<NamedFile> {
    let uuid = Uuid::from_str(&uuid).ok()?;

    NamedFile::open(config.analysis_pcap_file(&uuid)).await.ok()
}

#[utoipa::path(
        context_path = API_MOUNTPOINT,
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "Log file for the analysis", content_type = "application/octet-stream"),
            (status = 404, description = "Log file not found")
        ),
        tag = "analysis",
        description = "Retrieves the log file for a specific analysis by UUID."
    )]
#[get("/analysis/<uuid>/logs")]
async fn analysis_logs(uuid: &str, config: &State<Config>) -> Option<NamedFile> {
    let uuid = Uuid::from_str(uuid).ok()?;

    NamedFile::open(config.analysis_log_file(&uuid)).await.ok()
}

#[utoipa::path(
        context_path = API_MOUNTPOINT,
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "Graph file downloaded successfully", content_type = "application/octet-stream"),
            (status = 404, description = "Graph file not found")
        ),
        tag = "analysis",
        description = "Downloads a graph file associated with the given analysis UUID."
    )]
#[get("/analysis/<uuid>/graph")]
async fn analysis_graph(uuid: &str, config: &State<Config>) -> Option<NamedFile> {
    let uuid = Uuid::from_str(uuid).ok()?;

    NamedFile::open(config.analysis_graph_file(&uuid))
        .await
        .ok()
}

#[utoipa::path(
        params(
            ("uuid" = String, Path, description = "The UUID of the analysis")
        ),
        responses(
            (status = 200, description = "File downloaded successfully", content_type = "application/octet-stream"),
            (status = 404, description = "File not found")
        ),
        context_path = API_MOUNTPOINT,
        tag = "analysis",
        description = "Downloads a MISP event file associated with the given analysis UUID."
    )]
#[get("/analysis/<uuid>/misp-event")]
async fn analysis_misp_event(uuid: &str, config: &State<Config>) -> Option<NamedFile> {
    let uuid = Uuid::from_str(uuid).ok()?;

    NamedFile::open(config.analysis_misp_event_file(&uuid))
        .await
        .ok()
}

#[utoipa::path(
    context_path = API_MOUNTPOINT,
    responses(
        (status = 200, description = "List of available sandboxes", body = ApiResponse<Vec<config::Sandbox>>),
    ),
    tag = "sandboxes",
    description = "Retrieves a list of available sandboxes."
)]
#[get("/sandboxes/list")]
async fn sandbox_list(config: &State<Config>) -> ApiResult<&[config::Sandbox]> {
    Ok(ApiData::Some(&config.sandboxes))
}

#[get("/openapi/json")]
async fn openapi() -> ApiResult<utoipa::openapi::OpenApi> {
    Ok(ApiData::Some(ApiDoc::openapi()))
}

#[derive(Embed)]
#[folder = "../target/frontend"]
struct FrontendAssets;

// Catch-all route to serve index.html for Vue routes
#[get("/<path..>")]
async fn catch_all(path: PathBuf) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let filename = path.display().to_string();

    // if the asset exist we serve it
    if let Some(asset) = FrontendAssets::get(&filename) {
        let content_type = path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(ContentType::from_extension)
            .unwrap_or(ContentType::Bytes);
        Some((content_type, asset.data))
    } else {
        // if the asset doesn't exist we serve index.html
        // we delegate page routing to Vue
        let index = FrontendAssets::get("index.html")?;
        Some((ContentType::HTML, index.data))
    }
}

#[derive(Parser)]
struct Cli {
    /// Application configuration file
    #[arg(short, long, env = "APP_CONFIG")]
    config: Option<PathBuf>,

    /// Dump a configuration file example
    #[arg(long)]
    dump_config: bool,
}

#[derive(OpenApi)]
#[openapi(paths(
    analyze,
    analyze_again,
    analyses_search,
    analysis_status,
    analysis_metadata,
    analysis_sandbox,
    analysis_pcap,
    analysis_logs,
    analysis_graph,
    analysis_misp_event,
    sandbox_list
))]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let c = {
        let c: clap::Command = Cli::command();
        let styles = styling::Styles::styled()
            .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
            .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
            .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
            .placeholder(styling::AnsiColor::Cyan.on_default());

        c.styles(styles)
    };

    let cli: Cli =
        Cli::from_arg_matches(&c.get_matches()).map_err(|_| anyhow!("cli failed to parse args"))?;

    if cli.dump_config {
        let data_dir = PathBuf::from(".").join("app-data");
        let mut sbx_config = HashMap::new();
        sbx_config.insert(
            String::from("some-sandbox-name"),
            PathBuf::from("/path/to/configuration"),
        );
        let c = Config {
            kunai_sandbox_exe: PathBuf::from("change_me"),
            database: format!(
                "sqlite://{}",
                data_dir.join("database.sqlite3").to_string_lossy()
            ),
            sandboxes_config: sbx_config,
            default_sandbox_name: "some-sandbox-name".into(),
            data_dir,
            max_queue: 8,
            max_running: 4,
            ..Default::default()
        };

        println!(
            "{}",
            serde_yaml::to_string(&c).map_err(|_| anyhow!("failed at serializing config"))?
        );
        return Ok(());
    }

    let Some(config) = cli.config else {
        return Err(anyhow!("sandbox-ui needs a configuration file"));
    };

    let r = File::open(&config).map_err(|e| anyhow!("failed to open config file: {e}"))?;
    let api_config: Config = Config::from_reader(r)?;

    // configuration setup
    std::fs::create_dir_all(api_config.analyses_dir())
        .map_err(|e| anyhow!("failed to create analyses directory: {e}"))?;
    std::fs::create_dir_all(api_config.samples_dir())
        .map_err(|e| anyhow!("failed to create samples directory: {e}"))?;

    let db = db::set_up_db(&api_config.database).await?;
    let analyzer = Arc::new(Mutex::new(Analyzer::from_config_and_db(
        api_config.clone(),
        db,
    )));

    let c = analyzer.clone();
    tokio::spawn(Analyzer::run(c));

    let db = db::set_up_db(&api_config.database).await?;
    rocket::build()
        .configure(api_config.rocket.clone())
        .manage(analyzer)
        .manage(api_config)
        .manage(db)
        .mount(
            API_MOUNTPOINT,
            routes![
                openapi,
                analyze,
                analyze_again,
                analyses_search,
                analysis_status,
                analysis_metadata,
                analysis_sandbox,
                analysis_pcap,
                analysis_logs,
                analysis_graph,
                analysis_misp_event,
                sandbox_list
            ],
        )
        .mount("/", routes![catch_all]) // we handle view routes not to hit API with it
        .launch()
        .await?;

    Ok(())
}
