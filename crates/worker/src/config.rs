use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub token: String,
}

impl WorkerConfig {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn load(work_base: &PathBuf) -> Result<Option<Self>> {
        let config_path = Self::config_path(work_base);

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: WorkerConfig =
            serde_json::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(Some(config))
    }

    pub fn save(&self, work_base: &PathBuf) -> Result<()> {
        let config_path = Self::config_path(work_base);

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    fn config_path(work_base: &PathBuf) -> PathBuf {
        work_base.join("worker_config.json")
    }
}
