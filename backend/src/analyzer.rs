use crate::{Config, entities::analysis, entities::prelude::Analysis as DbAnalysis};
use chrono::{DateTime, Utc};
use log::error;
use md5::{Digest, Md5};
use sea_orm::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::{
    collections::HashMap,
    io::{self},
    net::IpAddr,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use utoipa::ToSchema;
use uuid::Uuid;

// WARNING: this structure is used in database
// so if it changes it might break things
#[derive(ToSchema)]
// to make it appear great in OpenAPI doc
#[allow(non_camel_case_types)]
pub enum AnalysisStatus {
    unqueued,
    queued,
    running,
    terminated,
    failed,
}

impl Serialize for AnalysisStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AnalysisStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| de::Error::custom("unknow variant"))
    }
}

impl std::fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AnalysisStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unqueued" => Ok(Self::unqueued),
            "queued" => Ok(Self::queued),
            "running" => Ok(Self::running),
            "terminated" => Ok(Self::terminated),
            "failed" => Ok(Self::failed),
            _ => Err(()),
        }
    }
}

impl AnalysisStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::unqueued => "unqueued",
            Self::queued => "queued",
            Self::running => "running",
            Self::terminated => "terminated",
            Self::failed => "failed",
        }
    }
}

// Custom serialization function for DateTime<Utc>
fn serialize_datetime<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            let formatted = date.format("%Y-%m-%d %H:%M:%S %z").to_string();
            serializer.serialize_str(&formatted)
        }
        None => serializer.serialize_none(),
    }
}

// Custom deserialization function for DateTime<Utc>
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(
            DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S %z")
                .map_err(de::Error::custom)?
                .to_utc(),
        )),
        None => Ok(None),
    }
}

#[derive(ToSchema, Default, Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub(crate) submission_name: Option<String>,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime"
    )]
    pub(crate) analysis_date: Option<DateTime<Utc>>,
    pub(crate) magic: Option<String>,
    pub(crate) md5: String,
    pub(crate) sha1: String,
    pub(crate) sha256: String,
    pub(crate) sha512: String,
    pub(crate) size: u64,
}

impl Metadata {
    fn from_reader<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let mut md5 = Md5::new();
        let mut sha1 = Sha1::new();
        let mut sha256 = Sha256::new();
        let mut sha512 = Sha512::new();
        let mut size = 0u64;

        let mut buf = [0; 4096];

        let mut n = reader.read(&mut buf[..])?;
        while n > 0 {
            md5.update(&buf[..n]);
            sha1.update(&buf[..n]);
            sha256.update(&buf[..n]);
            sha512.update(&buf[..n]);
            size = size.wrapping_add(n as u64);
            n = reader.read(&mut buf[..])?;
        }

        Ok(Self {
            md5: hex::encode(md5.finalize()),
            sha1: hex::encode(sha1.finalize()),
            sha256: hex::encode(sha256.finalize()),
            sha512: hex::encode(sha512.finalize()),
            size,
            ..Default::default()
        })
    }

    async fn from_async_reader<R: tokio::io::AsyncRead + Unpin>(mut reader: R) -> io::Result<Self> {
        let mut md5 = Md5::new();
        let mut sha1 = Sha1::new();
        let mut sha256 = Sha256::new();
        let mut sha512 = Sha512::new();
        let mut size = 0u64;

        let mut buf = [0; 4096];

        let mut n = reader.read(&mut buf[..]).await?;
        while n > 0 {
            md5.update(&buf[..n]);
            sha1.update(&buf[..n]);
            sha256.update(&buf[..n]);
            sha512.update(&buf[..n]);
            size = size.wrapping_add(n as u64);
            n = reader.read(&mut buf[..]).await?;
        }

        Ok(Self {
            md5: hex::encode(md5.finalize()),
            sha1: hex::encode(sha1.finalize()),
            sha256: hex::encode(sha256.finalize()),
            sha512: hex::encode(sha512.finalize()),
            size,
            ..Default::default()
        })
    }

    fn from_analysis(s: &Analysis) -> io::Result<Self> {
        let p = s.sample_path();

        let f = std::fs::File::open(p)?;

        Ok(Self {
            submission_name: s.submission_name.clone(),
            magic: None,
            ..Self::from_reader(f)?
        })
    }

    async fn from_analysis_async(s: &Analysis) -> tokio::io::Result<Self> {
        let p = s.sample_path();

        let f = tokio::fs::File::open(p).await?;

        Ok(Self {
            submission_name: s.submission_name.clone(),
            magic: None,
            ..Self::from_async_reader(f).await?
        })
    }

    fn analysis_date_now(mut self) -> Self {
        self.analysis_date = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Analysis {
    pub(crate) sandbox: String,
    pub(crate) sample_uuid: Uuid,
    pub(crate) sample_path: PathBuf,
    pub(crate) submission_name: Option<String>,
    pub(crate) analysis_uuid: Uuid,
    pub(crate) analysis_dir: PathBuf,
    pub(crate) metadata: Option<Metadata>,
    pub(crate) client_ip: IpAddr,
}

impl Analysis {
    pub const METADATA_FILENAME: &str = "metadata.json";
    pub const SANDBOX_FILENAME: &str = "sandbox.json";

    pub(crate) fn new(
        c: &Config,
        sandbox_name: Option<String>,
        sample_uuid: Uuid,
        submission_name: Option<String>,
        client_ip: IpAddr,
    ) -> Self {
        let analysis_uuid = Uuid::new_v4();
        let dir = c.analyses_dir().join(analysis_uuid.to_string());
        let sandbox = sandbox_name.unwrap_or(c.default_sandbox_name.clone());

        Analysis {
            sandbox,
            sample_uuid,
            sample_path: c.samples_dir().join(sample_uuid.to_string()),
            submission_name,
            analysis_uuid,
            analysis_dir: dir,
            metadata: None,
            client_ip,
        }
    }

    pub(crate) fn from_model_with_config(
        value: analysis::Model,
        c: &Config,
    ) -> Result<Self, AnalyzerError> {
        Ok(Self {
            sandbox: value.sandbox_name,
            sample_uuid: value.sample_uuid,
            sample_path: c.sample_path(&value.sample_uuid),
            submission_name: value.submission_name,
            analysis_uuid: value.uuid,
            analysis_dir: c.analysis_dir(&value.uuid),
            metadata: None,
            client_ip: IpAddr::from_str(&value.src_ip)
                .map_err(|e| AnalyzerError::msg(format!("failed to parse ip addr from db: {e}")))?,
        })
    }

    #[inline(always)]
    pub(crate) fn sample_path(&self) -> &PathBuf {
        &self.sample_path
    }

    #[inline(always)]
    pub(crate) fn metadata_json_file(&self) -> PathBuf {
        self.analysis_dir.join(Self::METADATA_FILENAME)
    }

    #[inline(always)]
    pub(crate) fn sandbox_json_file(&self) -> PathBuf {
        self.analysis_dir.join(Self::SANDBOX_FILENAME)
    }

    pub async fn async_metadata(&mut self) -> io::Result<&Metadata> {
        Ok(self
            .metadata
            .get_or_insert(Metadata::from_analysis_async(self).await?))
    }
}

struct RunningAnalysis {
    analysis: Analysis,
    handle: Option<tokio::task::JoinHandle<Result<(), AnalyzerError>>>,
    delete: bool,
}

pub struct Analyzer {
    config: Config,
    running: HashMap<Uuid, RunningAnalysis>,
    pub(crate) db: DatabaseConnection,
}

#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
    #[error("db: {0}")]
    Db(#[from] DbErr),
}

impl AnalyzerError {
    fn msg<S: AsRef<str>>(s: S) -> Self {
        Self::Msg(s.as_ref().into())
    }
}

impl Analyzer {
    pub fn from_config_and_db(value: Config, db: DatabaseConnection) -> Self {
        let running_capacity = value.max_running;

        Self {
            config: value,
            running: HashMap::with_capacity(running_capacity),
            db,
        }
    }

    pub(crate) async fn queue_new(&mut self, s: &Analysis) -> Result<bool, AnalyzerError> {
        let status = match self.is_queue_full().await {
            true => AnalysisStatus::unqueued,
            false => AnalysisStatus::queued,
        };

        // update database
        let a = analysis::ActiveModel {
            uuid: ActiveValue::Set(s.analysis_uuid),
            sandbox_name: ActiveValue::Set(s.sandbox.clone()),
            sample_uuid: ActiveValue::Set(s.sample_uuid),
            date: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            src_ip: ActiveValue::Set(s.client_ip.to_string()),
            submission_name: ActiveValue::Set(s.submission_name.clone()),
            status: ActiveValue::Set(status.to_string()),
        };

        a.insert(&self.db).await?;

        Ok(matches!(status, AnalysisStatus::queued))
    }

    pub(crate) async fn queue_existing(&mut self, s: &Analysis) -> Result<bool, AnalyzerError> {
        let status = match self.is_queue_full().await {
            true => AnalysisStatus::unqueued,
            false => AnalysisStatus::queued,
        };

        // update database
        let a = analysis::ActiveModel {
            uuid: ActiveValue::Set(s.analysis_uuid),
            sandbox_name: ActiveValue::Set(s.sandbox.clone()),
            sample_uuid: ActiveValue::Set(s.sample_uuid),
            date: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            src_ip: ActiveValue::Set(s.client_ip.to_string()),
            submission_name: ActiveValue::Set(s.submission_name.clone()),
            status: ActiveValue::Set(status.to_string()),
        };

        a.update(&self.db).await?;

        Ok(matches!(status, AnalysisStatus::queued))
    }

    #[inline(always)]
    pub(crate) fn is_running(&self, uuid: &Uuid) -> bool {
        self.running.contains_key(uuid)
    }

    async fn is_queue_full(&self) -> bool {
        let count = DbAnalysis::find()
            .filter(analysis::Column::Status.eq(AnalysisStatus::queued.to_string()))
            .count(&self.db)
            .await
            .unwrap_or_default();
        count >= self.config.max_queue as u64
    }

    async fn last_queued_not_running(&self) -> Result<Option<Analysis>, AnalyzerError> {
        let m = DbAnalysis::find()
            .filter(analysis::Column::Status.eq(AnalysisStatus::queued.to_string()))
            .order_by_desc(analysis::Column::Date)
            .all(&self.db)
            .await?
            .into_iter()
            .filter(|m| !self.running.contains_key(&m.uuid))
            .nth(0);

        let Some(m) = m else { return Ok(None) };

        Ok(Some(Analysis::from_model_with_config(m, &self.config)?))
    }

    pub(crate) async fn analyze_next(&mut self) -> Result<(), AnalyzerError> {
        if self.running.len() >= self.config.max_running {
            // there is no space for running applications
            return Ok(());
        }

        if let Some(analysis) = self.last_queued_not_running().await? {
            // FIXME: remove unwrap
            std::fs::create_dir_all(&analysis.analysis_dir).map_err(|e| {
                io::Error::other(format!("failed to create analysis directory: {e}"))
            })?;

            let a = analysis.clone();
            let sbx_exe = self.config.kunai_sandbox_exe.clone();

            let sandbox_name = if self.config.sandboxes_config.contains_key(&analysis.sandbox) {
                &analysis.sandbox
            } else {
                &self.config.default_sandbox_name
            };

            let sbx_config_path = self
                .config
                .sandboxes_config
                .get(sandbox_name)
                // should never happen because we have sanitized sandbox_name
                .expect("sandbox name must be found in config")
                .clone();

            let sandbox = self
                .config
                .sandboxes
                .iter()
                .filter(|s| &s.name == sandbox_name)
                .nth(0)
                // should never happen because we have sanitized sandbox_name
                .expect("sandbox name must be found in list")
                .clone();

            let analysis_dir = analysis.analysis_dir.join("analysis");

            let analyzer_stderr_path = a.analysis_dir.join("sandbox.stderr");

            let analyzer_stdout = std::fs::File::create(a.analysis_dir.join("sandbox.stdout"))
                .map_err(|e| {
                    io::Error::other(format!("failed to create sandbox.stdout file: {e}"))
                })?;
            let analyzer_stderr = std::fs::File::create(&analyzer_stderr_path).map_err(|e| {
                io::Error::other(format!("failed to create sandbox.stdout file: {e}"))
            })?;

            let h = tokio::spawn(async move {
                let analysis = a;

                let metadata = Metadata::from_analysis(&analysis)?.analysis_date_now();
                tokio::fs::write(
                    analysis.metadata_json_file(),
                    serde_json::to_string_pretty(&metadata).map_err(|e| {
                        io::Error::other(format!("failed to serialize metadata: {e}",))
                    })?,
                )
                .await
                .map_err(|e| io::Error::other(format!("failed to write metadata file: {e}")))?;

                tokio::fs::write(
                    analysis.sandbox_json_file(),
                    serde_json::to_string_pretty(&sandbox).map_err(|e| {
                        io::Error::other(format!("failed to serialize metadata: {e}",))
                    })?,
                )
                .await
                .map_err(|e| io::Error::other(format!("failed to write sandbox file: {e}")))?;

                // FIXME: remove this unwrap
                let mut child = Command::new(sbx_exe)
                    // we need to force re-analysis in case we stopped app while analysis was running
                    .arg("--force")
                    .arg("-t")
                    .arg("60")
                    .arg("--config")
                    .arg(sbx_config_path)
                    .arg("--output-dir")
                    .arg(analysis_dir)
                    .arg("--no-dropped")
                    .arg("--tmp")
                    .arg("--graph")
                    .arg("--misp")
                    .arg("--sync-time")
                    .arg("--compress")
                    .arg("--")
                    .arg(analysis.sample_path())
                    .stdout(analyzer_stdout)
                    .stderr(analyzer_stderr)
                    .spawn()?;

                while let Ok(None) = child.try_wait() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                let status = child.wait()?;

                if !status.success() {
                    return Err(AnalyzerError::Msg(format!(
                        "analysis failed, inspect {}",
                        analyzer_stderr_path.to_string_lossy(),
                    )));
                }

                Ok(())
            });

            let _ = self
                .running
                .entry(analysis.analysis_uuid)
                .or_insert_with(|| RunningAnalysis {
                    analysis,
                    handle: Some(h),
                    delete: false,
                });
        }

        Ok(())
    }

    pub(crate) async fn analysis_status(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<AnalysisStatus>, AnalyzerError> {
        if self.running.contains_key(uuid) {
            return Ok(Some(AnalysisStatus::running));
        }

        let Some(status_str) = DbAnalysis::find_by_id(*uuid)
            .one(&self.db)
            .await?
            .map(|m| m.status)
        else {
            return Ok(None);
        };

        let status = AnalysisStatus::from_str(&status_str)
            .map_err(|_| AnalyzerError::msg("failed to convert status str from db"))?;

        Ok(Some(status))
    }

    pub(crate) async fn run(analyzer: Arc<Mutex<Self>>) {
        loop {
            {
                let mut analyzer = analyzer.lock().await;

                let _ = analyzer
                    .analyze_next()
                    .await
                    .inspect_err(|e| error!("failed at starting analysis: {e}"));

                let mut inserts = vec![];

                for (u, ra) in analyzer
                    .running
                    .iter_mut()
                    // we take only finished analysis
                    .filter(|(_, ra)| {
                        if let Some(h) = ra.handle.as_ref() {
                            h.is_finished()
                        } else {
                            false
                        }
                    })
                {
                    let mut status = AnalysisStatus::terminated;
                    if let Some(handle) = ra.handle.take() {
                        if let Ok(Err(err)) = handle
                            .await
                            .inspect_err(|_| error!("analysis={u} failed to join analyzer thread"))
                        {
                            error!("sandbox analysis={u} failed: {err}");
                            status = AnalysisStatus::failed
                        }
                    }

                    info!("analysis={u} finished with status: {}", status);

                    // update database
                    let a = analysis::ActiveModel {
                        uuid: ActiveValue::Set(ra.analysis.analysis_uuid),
                        sandbox_name: ActiveValue::Set(ra.analysis.sandbox.clone()),
                        sample_uuid: ActiveValue::Set(ra.analysis.sample_uuid),
                        date: ActiveValue::Set(chrono::Utc::now().naive_utc()),
                        src_ip: ActiveValue::Set(ra.analysis.client_ip.to_string()),
                        submission_name: ActiveValue::Set(ra.analysis.submission_name.clone()),
                        status: ActiveValue::Set(status.to_string()),
                    };

                    inserts.push(a);

                    // running analysis can be deleted
                    ra.delete = true
                }

                // we do all the inserts in the db
                for update in inserts {
                    let _ = update.update(&analyzer.db).await.inspect_err(|e| {
                        error!("failed to update analysis record in database:{e}")
                    });
                }

                // we remove running analysis that can be dropped
                analyzer.running.retain(|_, ra| !ra.delete);
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
