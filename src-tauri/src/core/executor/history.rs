use std::collections::VecDeque;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub connection_id: String,
    pub sql: String,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub affected_rows: u64,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionStatus {
    Success,
    Error(String),
}

pub struct ExecutionHistory {
    entries: RwLock<VecDeque<HistoryEntry>>,
    max_entries: usize,
}

impl ExecutionHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(max_entries)),
            max_entries,
        }
    }

    pub async fn add(&self, entry: HistoryEntry) {
        let mut entries = self.entries.write().await;
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub async fn list(&self) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().cloned().collect()
    }

    pub async fn list_by_connection(&self, conn_id: &str) -> Vec<HistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter()
            .filter(|e| e.connection_id == conn_id)
            .cloned()
            .collect()
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}
