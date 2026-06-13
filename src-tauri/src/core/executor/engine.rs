use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::core::connection::manager::ConnectionManager;
use crate::core::executor::history::{ExecutionHistory, HistoryEntry, ExecutionStatus};
use crate::core::plugin::registry::PluginRegistry;
use crate::core::types::QueryResult;
use crate::errors::AppError;

pub struct SqlEngine {
    connection_manager: Arc<ConnectionManager>,
    plugin_registry: Arc<PluginRegistry>,
    history: Arc<ExecutionHistory>,
    running_queries: RwLock<HashMap<String, bool>>,
}

impl SqlEngine {
    pub fn new(connection_manager: Arc<ConnectionManager>, plugin_registry: Arc<PluginRegistry>, history: Arc<ExecutionHistory>) -> Self {
        Self {
            connection_manager,
            plugin_registry,
            history,
            running_queries: RwLock::new(HashMap::new()),
        }
    }

    pub async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult, AppError> {
        let query_id = Uuid::new_v4().to_string();
        {
            let mut running = self.running_queries.write().await;
            running.insert(query_id.clone(), true);
        }

        let config = self.connection_manager.pool().get_config(conn_id).await?;
        let driver = self.plugin_registry.get_driver(&config.db_type).await?;

        let start = std::time::Instant::now();
        let result = driver.execute(conn_id, sql).await;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        {
            let mut running = self.running_queries.write().await;
            running.remove(&query_id);
        }

        match result {
            Ok(query_result) => {
                self.record_history(conn_id, sql, execution_time_ms, query_result.affected_rows, ExecutionStatus::Success).await;
                Ok(query_result)
            }
            Err(e) => {
                self.record_history(conn_id, sql, execution_time_ms, 0, ExecutionStatus::Error(e.to_string())).await;
                Err(e)
            }
        }
    }

    pub async fn record_history(
        &self,
        conn_id: &str,
        sql: &str,
        execution_time_ms: u64,
        affected_rows: u64,
        status: ExecutionStatus,
    ) {
        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            connection_id: conn_id.to_string(),
            sql: sql.to_string(),
            executed_at: chrono::Utc::now(),
            execution_time_ms,
            affected_rows,
            status,
        };
        self.history.add(entry).await;
    }

    pub async fn cancel_query(&self, query_id: &str) -> Result<(), AppError> {
        let mut running = self.running_queries.write().await;
        running.remove(query_id);
        Ok(())
    }

    pub fn history(&self) -> Arc<ExecutionHistory> {
        self.history.clone()
    }
}
