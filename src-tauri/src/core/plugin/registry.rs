use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::plugin::driver::DatabaseDriver;
use crate::core::types::DatabaseType;
use crate::errors::AppError;

pub struct PluginRegistry {
    drivers: RwLock<HashMap<DatabaseType, Arc<dyn DatabaseDriver>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            drivers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, driver: Arc<dyn DatabaseDriver>) -> Result<(), AppError> {
        let info = driver.info();
        let mut drivers = self.drivers.write().await;
        drivers.insert(info.db_type, driver);
        Ok(())
    }

    pub async fn get_driver(&self, db_type: &DatabaseType) -> Result<Arc<dyn DatabaseDriver>, AppError> {
        let drivers = self.drivers.read().await;
        drivers.get(db_type).cloned().ok_or_else(|| {
            AppError::Plugin(format!("No driver registered for {:?}", db_type))
        })
    }

    pub async fn list_drivers(&self) -> Vec<crate::core::types::DriverInfo> {
        let drivers = self.drivers.read().await;
        drivers.values().map(|d| d.info()).collect()
    }

    pub async fn unregister(&self, db_type: &DatabaseType) -> Result<(), AppError> {
        let mut drivers = self.drivers.write().await;
        drivers.remove(db_type).ok_or_else(|| {
            AppError::Plugin(format!("No driver registered for {:?}", db_type))
        })?;
        Ok(())
    }
}
