pub mod connection;
pub mod query;
pub mod metadata;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to HHL-DMA.", name)
}
