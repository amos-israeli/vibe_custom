use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::sync::Mutex;

// Store for registered shortcuts
pub struct HotkeyManager {
    registered_shortcuts: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self {
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub start_recording_hotkey: String,
    pub stop_recording_hotkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyError {
    pub error: String,
    pub needs_permission: bool,
}

#[tauri::command]
pub async fn register_hotkeys(
    app_handle: AppHandle,
    config: HotkeyConfig,
) -> Result<(), HotkeyError> {
    if !config.enabled {
        return Ok(());
    }

    let hotkey_manager = app_handle.state::<HotkeyManager>();
    let mut registered = hotkey_manager.registered_shortcuts.lock().await;

    // Unregister existing shortcuts first
    for shortcut_str in registered.values() {
        if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
            let _ = app_handle.global_shortcut().unregister(shortcut);
        }
    }
    registered.clear();

    // Register start recording hotkey
    match register_single_hotkey(
        &app_handle,
        &config.start_recording_hotkey,
        "start_recording",
    ).await {
        Ok(()) => {
            registered.insert("start_recording".to_string(), config.start_recording_hotkey.clone());
        }
        Err(e) => {
            tracing::warn!("Failed to register start recording hotkey: {}", e);
            return Err(HotkeyError {
                error: format!("Failed to register start recording hotkey '{}': {}. This may be because the key combination is already in use by another application or the system.", config.start_recording_hotkey, e),
                needs_permission: false,
            });
        }
    }

    // Register stop recording hotkey
    match register_single_hotkey(
        &app_handle,
        &config.stop_recording_hotkey,
        "stop_recording",
    ).await {
        Ok(()) => {
            registered.insert("stop_recording".to_string(), config.stop_recording_hotkey.clone());
        }
        Err(e) => {
            tracing::warn!("Failed to register stop recording hotkey: {}", e);
            return Err(HotkeyError {
                error: format!("Failed to register stop recording hotkey '{}': {}. This may be because the key combination is already in use by another application or the system.", config.stop_recording_hotkey, e),
                needs_permission: false,
            });
        }
    }

    tracing::info!("Successfully registered hotkeys: start='{}', stop='{}'", 
                   config.start_recording_hotkey, config.stop_recording_hotkey);
    
    Ok(())
}

async fn register_single_hotkey(
    app_handle: &AppHandle,
    shortcut_str: &str,
    _action: &str,
) -> Result<()> {
    let shortcut = shortcut_str.parse::<Shortcut>()
        .map_err(|e| eyre::eyre!("Invalid shortcut format '{}': {}", shortcut_str, e))?;

    app_handle.global_shortcut().register(shortcut)?;

    Ok(())
}

#[tauri::command]
pub async fn unregister_hotkeys(app_handle: AppHandle) -> Result<(), String> {
    let hotkey_manager = app_handle.state::<HotkeyManager>();
    let mut registered = hotkey_manager.registered_shortcuts.lock().await;

    for shortcut_str in registered.values() {
        if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
            if let Err(e) = app_handle.global_shortcut().unregister(shortcut) {
                tracing::warn!("Failed to unregister shortcut '{}': {}", shortcut_str, e);
            }
        }
    }
    
    registered.clear();
    tracing::info!("All hotkeys unregistered");
    
    Ok(())
}

#[tauri::command]
pub async fn check_hotkey_availability(shortcut_str: String) -> Result<bool, String> {
    match shortcut_str.parse::<Shortcut>() {
        Ok(_) => Ok(true), // Valid format
        Err(e) => Err(format!("Invalid shortcut format: {}", e)),
    }
}

#[tauri::command]
pub async fn get_registered_hotkeys(app_handle: AppHandle) -> Result<HashMap<String, String>, String> {
    let hotkey_manager = app_handle.state::<HotkeyManager>();
    let registered = hotkey_manager.registered_shortcuts.lock().await;
    Ok(registered.clone())
}