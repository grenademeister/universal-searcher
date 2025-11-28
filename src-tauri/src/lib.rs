use std::env;
use std::path::PathBuf;
use std::process::Command;

use tauri::path::BaseDirectory;
use tauri::Manager;

#[tauri::command]
async fn run_overlay_cli(app: tauri::AppHandle, provider: Option<String>) -> Result<String, String> {
    let cli = resolve_cli_path(&app);

    let output = tauri::async_runtime::spawn_blocking(move || {
        let mut command = Command::new(&cli);
        if let Some(p) = provider {
            command.env("OVERLAY_PROVIDER", p);
        }

        let output = command
            .output()
            .map_err(|err| format!("failed to start {cli}: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = stderr.trim();
            if msg.is_empty() {
                Err(format!("{cli} exited with {}", output.status))
            } else {
                Err(msg.to_string())
            }
        } else {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
    })
    .await
    .map_err(|err| format!("failed to run overlay-cli: {err}"))??;

    Ok(output)
}

fn resolve_cli_path(app: &tauri::AppHandle) -> String {
    if let Ok(path) = env::var("OVERLAY_CLI_PATH") {
        return path;
    }

    if let Ok(path) = app
        .path()
        .resolve("overlay-cli", BaseDirectory::Resource)
    {
        if path.exists() {
            if let Some(path_str) = path.to_str() {
                return path_str.to_string();
            }
        }
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            for name in ["overlay-cli", "overlay-cli.exe"] {
                let candidate: PathBuf = dir.join(name);
                if candidate.exists() {
                    if let Some(path_str) = candidate.to_str() {
                        return path_str.to_string();
                    }
                }
            }
        }
    }

    "overlay-cli".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_overlay_cli])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
