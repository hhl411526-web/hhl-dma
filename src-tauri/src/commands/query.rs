use tauri::State;
use std::sync::Arc;
use crate::core::executor::engine::SqlEngine;
use crate::core::executor::result::ResultProcessor;
use crate::core::types::QueryResult;
use crate::errors::AppError;

#[tauri::command]
pub async fn execute_query(
    engine: State<'_, Arc<SqlEngine>>,
    conn_id: String,
    sql: String,
) -> Result<QueryResult, AppError> {
    engine.execute(&conn_id, &sql).await
}

#[tauri::command]
pub async fn execute_query_paginated(
    engine: State<'_, Arc<SqlEngine>>,
    conn_id: String,
    sql: String,
    page: usize,
    page_size: usize,
) -> Result<crate::core::executor::result::PaginatedResult, AppError> {
    let result = engine.execute(&conn_id, &sql).await?;
    Ok(ResultProcessor::paginate(&result, page, page_size))
}

#[tauri::command]
pub async fn cancel_query(
    engine: State<'_, Arc<SqlEngine>>,
    query_id: String,
) -> Result<(), AppError> {
    engine.cancel_query(&query_id).await
}

#[tauri::command]
pub async fn get_query_history(
    engine: State<'_, Arc<SqlEngine>>,
    conn_id: Option<String>,
) -> Result<Vec<crate::core::executor::history::HistoryEntry>, AppError> {
    let history = engine.history();
    let entries = if let Some(conn_id) = conn_id {
        history.list_by_connection(&conn_id).await
    } else {
        history.list().await
    };
    Ok(entries)
}
