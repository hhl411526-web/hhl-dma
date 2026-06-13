use std::path::PathBuf;
use crate::core::types::ConnectionConfig;
use crate::core::security::crypto::{encrypt_password, decrypt_password};
use crate::errors::AppError;

pub struct ConnectionConfigStore {
    config_dir: PathBuf,
    master_password: Option<String>,
}

impl ConnectionConfigStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            master_password: None,
        }
    }

    pub fn set_master_password(&mut self, password: String) {
        self.master_password = Some(password);
    }

    pub async fn save(&self, config: &ConnectionConfig) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.config_dir).map_err(AppError::Io)?;
        let mut config = config.clone();
        if !config.password.is_empty() {
            config.password = encrypt_password(&config.password, self.master_password.as_deref())?;
        }
        let file_path = self.config_dir.join(format!("{}.json", config.id));
        let json = serde_json::to_string_pretty(&config).map_err(AppError::Serialization)?;
        std::fs::write(file_path, json).map_err(AppError::Io)?;
        Ok(())
    }

    pub async fn load_all(&self) -> Result<Vec<ConnectionConfig>, AppError> {
        if !self.config_dir.exists() {
            return Ok(Vec::new());
        }
        let mut configs = Vec::new();
        let entries = std::fs::read_dir(&self.config_dir).map_err(AppError::Io)?;
        for entry in entries {
            let entry = entry.map_err(AppError::Io)?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = std::fs::read_to_string(&path).map_err(AppError::Io)?;
                let mut config: ConnectionConfig = serde_json::from_str(&content).map_err(AppError::Serialization)?;
                if !config.password.is_empty() {
                    config.password = decrypt_password(&config.password, self.master_password.as_deref())?;
                }
                configs.push(config);
            }
        }
        Ok(configs)
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let file_path = self.config_dir.join(format!("{}.json", id));
        if file_path.exists() {
            std::fs::remove_file(file_path).map_err(AppError::Io)?;
        }
        Ok(())
    }
}
