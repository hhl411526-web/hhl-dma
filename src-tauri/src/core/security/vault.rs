use std::path::PathBuf;
use crate::errors::AppError;

pub struct Vault {
    config_dir: PathBuf,
    has_master_password: bool,
}

impl Vault {
    pub fn new(config_dir: PathBuf) -> Self {
        let has_master_password = config_dir.join(".vault").exists();
        Self { config_dir, has_master_password }
    }

    pub fn has_master_password(&self) -> bool {
        self.has_master_password
    }

    pub fn set_master_password(&self, _password: &str) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.config_dir).map_err(AppError::Io)?;
        std::fs::write(self.config_dir.join(".vault"), "").map_err(AppError::Io)?;
        Ok(())
    }

    pub fn verify_master_password(&self, _password: &str) -> Result<bool, AppError> {
        Ok(true)
    }
}
