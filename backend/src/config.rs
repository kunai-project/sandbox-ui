use std::{collections::HashMap, fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::analyzer::Analysis;
use anyhow::anyhow;
use serde::de::Error;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Config {
    pub(crate) database: String,
    pub(crate) kunai_sandbox_exe: PathBuf,
    pub(crate) sandboxes_config: HashMap<String, PathBuf>,
    pub(crate) default_sandbox_name: String,
    pub(crate) data_dir: PathBuf,
    pub(crate) max_queue: usize,
    pub(crate) max_running: usize,
    #[serde(skip)]
    pub(crate) sandboxes: Vec<Sandbox>,
}

impl Config {
    pub(crate) fn from_reader<R: std::io::Read>(r: R) -> anyhow::Result<Self> {
        let mut config: Config = serde_yaml::from_reader(r)?;

        let mut sbx = vec![];

        // always process default config first so that it appears on top of the list
        let p = config
            .sandboxes_config
            .get(&config.default_sandbox_name)
            .ok_or(anyhow!(
                "default sandbox name must be one of the configured sandboxes"
            ))?;

        sbx.push(
            Sandbox::from_reader_with_name(
                config.default_sandbox_name.clone(),
                File::open(p)
                    .map_err(|e| anyhow!("failed to open sandbox configuration file: {e}"))?,
            )
            .map_err(|e| anyhow!("failed to retrieve sandbox information from config: {e}"))?,
        );

        // process all other sandboxes
        for (name, p) in config
            .sandboxes_config
            .iter()
            // we filter out sandboxes not having the default name
            .filter(|(name, _)| *name != &config.default_sandbox_name)
        {
            sbx.push(
                Sandbox::from_reader_with_name(
                    name.clone(),
                    File::open(p)
                        .map_err(|e| anyhow!("failed to open sandbox configuration file: {e}"))?,
                )
                .map_err(|e| anyhow!("failed to retrieve sandbox information from config: {e}"))?,
            );
        }

        config.sandboxes = sbx;

        Ok(config)
    }

    pub(crate) fn analyses_dir(&self) -> PathBuf {
        self.data_dir.join("analyses")
    }

    pub(crate) fn samples_dir(&self) -> PathBuf {
        self.data_dir.join("samples")
    }

    pub(crate) fn sample_path(&self, sample_uuid: &Uuid) -> PathBuf {
        self.samples_dir().join(sample_uuid.to_string())
    }

    #[inline]
    pub(crate) fn sample_metadata_json_file(&self, uuid: &Uuid) -> PathBuf {
        self.analyses_dir()
            .join(uuid.to_string())
            .join(Analysis::METADATA_FILENAME)
    }

    #[inline]
    pub(crate) fn analysis_sandbox_json_file(&self, uuid: &Uuid) -> PathBuf {
        self.analyses_dir()
            .join(uuid.to_string())
            .join(Analysis::SANDBOX_FILENAME)
    }

    #[inline(always)]
    pub(crate) fn analysis_dir(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.analyses_dir().join(analysis_uuid.to_string())
    }

    #[inline(always)]
    pub(crate) fn sandbox_analysis_dir(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.analysis_dir(analysis_uuid).join("analysis")
    }

    #[inline]
    pub(crate) fn analysis_pcap_file(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.sandbox_analysis_dir(analysis_uuid).join("dump.pcap")
    }

    #[inline]
    pub(crate) fn analysis_log_file(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.sandbox_analysis_dir(analysis_uuid)
            .join("kunai.jsonl.gz")
    }

    #[inline]
    pub(crate) fn analysis_graph_file(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.sandbox_analysis_dir(analysis_uuid).join("graph.svg")
    }

    #[inline]
    pub(crate) fn analysis_misp_event_file(&self, analysis_uuid: &Uuid) -> PathBuf {
        self.sandbox_analysis_dir(analysis_uuid)
            .join("misp-event.json")
    }
}

#[derive(ToSchema, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Sandbox {
    pub(crate) name: String,
    pub(crate) arch: String,
    pub(crate) kernel: String,
    pub(crate) distribution: String,
}

impl Sandbox {
    pub(crate) fn from_reader_with_name<R: std::io::Read>(
        name: String,
        r: R,
    ) -> serde_yaml::Result<Self> {
        let v: serde_yaml::Value = serde_yaml::from_reader(r)?;
        let qemu = v.get("qemu").ok_or(serde_yaml::Error::custom(
            "missing qemu key in sandbox config",
        ))?;

        let distro =
            qemu.get("distribution")
                .and_then(|v| v.as_str())
                .ok_or(serde_yaml::Error::custom(
                    "missing distribution information in config",
                ))?;

        if distro.is_empty() {
            return Err(serde_yaml::Error::custom(
                "distribution field isn't allowed to be empty",
            ));
        }

        let arch = qemu
            .get("arch")
            .and_then(|v| v.as_str())
            .ok_or(serde_yaml::Error::custom(
                "missing architecture information in config",
            ))?;

        if arch.is_empty() {
            return Err(serde_yaml::Error::custom(
                "arch field isn't allowed to be empty",
            ));
        }

        let kernel =
            qemu.get("kernel")
                .and_then(|v| v.as_str())
                .ok_or(serde_yaml::Error::custom(
                    "missing kernel information in config",
                ))?;

        if kernel.is_empty() {
            return Err(serde_yaml::Error::custom(
                "kernel field isn't allowed to be empty",
            ));
        }

        Ok(Sandbox {
            name,
            arch: arch.to_ascii_lowercase(),
            kernel: kernel.to_ascii_lowercase(),
            distribution: distro.to_ascii_lowercase(),
        })
    }
}
