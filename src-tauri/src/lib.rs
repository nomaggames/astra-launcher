mod updater;
mod game_runner;
mod config;

use tauri::Emitter;
use updater::{check_for_updates, download_update, get_installed_version, UpdateInfo, DownloadProgress};
use game_runner::launch_game;
use config::{LauncherConfig, load_config, save_config};

#[tauri::command]
async fn check_updates() -> Result<UpdateInfo, String> {
    check_for_updates().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_current_version() -> Result<Option<String>, String> {
    get_installed_version().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_game_update(
    version: String,
    download_url: String,
    window: tauri::Window,
) -> Result<(), String> {
    download_update(&version, &download_url, move |progress: DownloadProgress| {
        let _ = window.emit("download-progress", &progress);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_astra() -> Result<(), String> {
    launch_game().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn uninstall_game() -> Result<(), String> {
    updater::uninstall_game().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config() -> Result<LauncherConfig, String> {
    load_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_config(config: LauncherConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            check_updates,
            get_current_version,
            download_game_update,
            launch_astra,
            uninstall_game,
            get_config,
            update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
