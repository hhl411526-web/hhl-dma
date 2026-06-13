use crate::core::types::QueryResult;

pub struct ResultProcessor;

impl ResultProcessor {
    pub fn paginate(result: &QueryResult, page: usize, page_size: usize) -> PaginatedResult {
        let total_rows = result.rows.len();
        let total_pages = (total_rows + page_size - 1) / page_size;
        let start = page * page_size;
        let end = std::cmp::min(start + page_size, total_rows);
        let page_rows = if start < total_rows {
            result.rows[start..end].to_vec()
        } else {
            Vec::new()
        };
        PaginatedResult {
            columns: result.columns.clone(),
            rows: page_rows,
            page,
            page_size,
            total_rows,
            total_pages,
            execution_time_ms: result.execution_time_ms,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaginatedResult {
    pub columns: Vec<crate::core::types::ColumnDef>,
    pub rows: Vec<serde_json::Value>,
    pub page: usize,
    pub page_size: usize,
    pub total_rows: usize,
    pub total_pages: usize,
    pub execution_time_ms: u64,
}
