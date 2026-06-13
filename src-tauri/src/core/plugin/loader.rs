use std::sync::Arc;
use crate::core::plugin::registry::PluginRegistry;
use crate::errors::AppError;

pub struct PluginLoader {
    registry: Arc<PluginRegistry>,
}

impl PluginLoader {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }

    pub async fn load_builtin_plugins(&self) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn load_external_plugins(&self, _plugins_dir: &str) -> Result<(), AppError> {
        Ok(())
    }
}
