// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cleaner;
mod cli;
mod cmd;
mod config;
mod panic_hook;

#[cfg(feature = "server")]
mod server;

mod setup;
mod utils;
use tauri::{Emitter, Manager};
mod logging;

#[cfg(target_os = "macos")]
mod dock;

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows"))]
mod x86_features;

#[cfg(windows)]
mod custom_protocol;

#[cfg(windows)]
mod gpu_preference;

#[cfg(target_os = "macos")]
mod screen_capture_kit;

use eyre::{eyre, Result};
use tauri_plugin_window_state::StateFlags;

use utils::LogError;

fn main() -> Result<()> {
    // Attach console in Windows:
    #[cfg(all(windows, not(debug_assertions)))]
    cli::attach_console();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Create app directories
            let local_app_data_dir = app.path().app_local_data_dir()?;
            let app_config_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&local_app_data_dir)?;
            std::fs::create_dir_all(&app_config_dir)?;
            
            // Create main window only in non-CLI mode
            if !crate::cli::is_cli_detected() {
                let _window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .inner_size(1200.0, 800.0)
                    .min_inner_size(800.0, 700.0)
                    .center()
                    .title("Vibe")
                    .resizable(true)
                    .focused(true)
                    .visible(true)
                    .build()?;
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::hotkey::register_hotkeys,
            cmd::hotkey::unregister_hotkeys,
            cmd::hotkey::check_hotkey_availability,
            cmd::hotkey::get_registered_hotkeys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
