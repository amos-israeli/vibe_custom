use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Simple hotkey manager - actual registration is handled by frontend
pub struct HotkeyManager;

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
pub async fn register_hotkeys(_config: HotkeyConfig) -> Result<(), HotkeyError> {
    // Registration is now handled in frontend with @tauri-apps/plugin-global-shortcut
    Ok(())
}

#[tauri::command]
pub async fn unregister_hotkeys() -> Result<(), String> {
    // Unregistration is now handled in frontend with @tauri-apps/plugin-global-shortcut
    Ok(())
}

#[tauri::command]
pub async fn check_hotkey_availability(shortcut_str: String) -> Result<bool, String> {
    // Basic format validation - actual availability checking is done in frontend
    if shortcut_str.trim().is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }
    Ok(true)
}

#[tauri::command]
pub async fn get_registered_hotkeys() -> Result<HashMap<String, String>, String> {
    // This is now handled in frontend
    Ok(HashMap::new())
}