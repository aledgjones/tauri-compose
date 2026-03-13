mod api;
mod mcp;
mod registry;

use tauri::AppHandle;

pub const DEFAULT_AUTOMATION_HOST: &str = "127.0.0.1";
pub const DEFAULT_AUTOMATION_PORT: u16 = 4767;

pub fn start_server(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = api::serve(app_handle).await {
            eprintln!("compose automation api failed: {error}");
        }
    });
}
