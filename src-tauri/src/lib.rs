mod config;
mod overlay;

use tauri::{Manager, PhysicalSize, Size};

#[derive(Clone)]
struct ManagedConfig(config::AppConfig);

#[tauri::command]
async fn generate_overlay(
    provider: Option<String>,
    model: Option<String>,
) -> Result<overlay::OverlayResponse, String> {
    overlay::generate(provider, model).await
}

#[tauri::command]
fn shutdown(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn load_config(state: tauri::State<ManagedConfig>) -> config::AppConfig {
    state.0.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_config = config::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup({
            let cfg = app_config.clone();
            move |app| {
                if let Some(window) = app.get_webview_window("main") {
                    let size = Size::Physical(PhysicalSize::new(cfg.window.width, cfg.window.height));
                    let _ = window.set_size(size);
                }
                Ok(())
            }
        })
        .manage(ManagedConfig(app_config))
        .invoke_handler(tauri::generate_handler![
            generate_overlay,
            shutdown,
            load_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
