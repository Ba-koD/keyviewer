use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub port: u16,
    pub language: String,
    pub run_on_startup: bool,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            port: 8000,
            language: "ko".to_string(),
            run_on_startup: false,
        }
    }
}

impl LauncherSettings {
    // Detect if running as portable (check for "portable" in exe name)
    fn is_portable() -> bool {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_name) = exe_path.file_name() {
                return exe_name.to_string_lossy().to_lowercase().contains("portable");
            }
        }
        false
    }

    // Get settings file path (different for portable vs installed)
    pub fn get_settings_path() -> PathBuf {
        if Self::is_portable() {
            // Portable: save settings next to exe
            let mut path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            path.push("keyviewer-portable.json");
            path
        } else {
            // Installed: use config directory
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("keyviewer");
            fs::create_dir_all(&path).ok();
            path.push("settings.json");
            path
        }
    }

    // Load settings from file
    pub fn load() -> Self {
        let path = Self::get_settings_path();
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    // Save settings to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_settings_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))?;
        Ok(())
    }
}

// Windows startup registry functions
#[cfg(target_os = "windows")]
pub fn set_windows_startup(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        key.set_value("KeyViewer", &exe_path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        key.delete_value("KeyViewer").ok();
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_windows_startup(_enabled: bool) -> Result<(), String> {
    Err("Not supported on this platform".to_string())
}

