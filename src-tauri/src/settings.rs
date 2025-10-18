use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub port: u16,
    pub language: String,
    pub run_on_startup: bool,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            port: 8000, // Default port, always starts with this
            language: "ko".to_string(), // Default language, always starts with Korean
            run_on_startup: false, // Default disabled, always starts disabled
        }
    }
}

impl LauncherSettings {
    // Load settings from Windows Registry (no file on disk!)
    pub fn load() -> Self {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key_path = r"Software\KeyViewer";
            
            if let Ok(key) = hkcu.open_subkey(key_path) {
                let port_u32: u32 = key.get_value("Port").unwrap_or(8000);
                let port = port_u32 as u16;
                let language: String = key.get_value("Language").unwrap_or_else(|_| "ko".to_string());
                let run_on_startup: u32 = key.get_value("RunOnStartup").unwrap_or(0);
                
                return Self {
                    port,
                    language,
                    run_on_startup: run_on_startup != 0,
                };
            }
        }
        
        // Fallback to defaults if registry read fails or not Windows
        Self::default()
    }

    // Save settings to Windows Registry (no file on disk!)
    pub fn save(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key_path = r"Software\KeyViewer";
            
            // Create the key if it doesn't exist
            let (key, _) = hkcu.create_subkey(key_path)
                .map_err(|e| format!("Failed to create registry key: {}", e))?;
            
            // Save settings to registry (port as u32 because registry doesn't support u16)
            let port_u32: u32 = self.port as u32;
            key.set_value("Port", &port_u32)
                .map_err(|e| format!("Failed to save port: {}", e))?;
            key.set_value("Language", &self.language)
                .map_err(|e| format!("Failed to save language: {}", e))?;
            let run_on_startup_u32: u32 = if self.run_on_startup { 1 } else { 0 };
            key.set_value("RunOnStartup", &run_on_startup_u32)
                .map_err(|e| format!("Failed to save run_on_startup: {}", e))?;
            
            return Ok(());
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // On non-Windows platforms, just return Ok (settings won't persist)
            Ok(())
        }
    }
}

// Windows startup registry functions
#[cfg(target_os = "windows")]
pub fn set_windows_startup(enabled: bool) -> Result<(), String> {
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

// Save/Load Target Config to/from Registry
pub fn save_target_config(mode: &str, value: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(r"Software\KeyViewer")
            .map_err(|e| format!("Failed to create registry key: {}", e))?;
        
        key.set_value("TargetMode", &mode.to_string())
            .map_err(|e| format!("Failed to save target mode: {}", e))?;
        key.set_value("TargetValue", &value.unwrap_or(""))
            .map_err(|e| format!("Failed to save target value: {}", e))?;
        
        return Ok(());
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

pub fn load_target_config() -> (String, Option<String>) {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\KeyViewer") {
            let mode: String = key.get_value("TargetMode").unwrap_or_else(|_| "disabled".to_string());
            let value: String = key.get_value("TargetValue").unwrap_or_else(|_| "".to_string());
            let value_opt = if value.is_empty() { None } else { Some(value) };
            return (mode, value_opt);
        }
    }
    
    // Default values
    ("disabled".to_string(), None)
}

// Save/Load Overlay Config to/from Registry
pub fn save_overlay_config(
    fade_in_ms: u32,
    fade_out_ms: u32,
    chip_bg: &str,
    chip_fg: &str,
    chip_gap: u32,
    chip_pad_v: u32,
    chip_pad_h: u32,
    chip_radius: u32,
    chip_font_px: u32,
    chip_font_weight: u32,
    background: &str,
    cols: u32,
    rows: u32,
    align: &str,
    direction: &str,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(r"Software\KeyViewer\Overlay")
            .map_err(|e| format!("Failed to create registry key: {}", e))?;
        
        // Save all overlay settings
        key.set_value("FadeInMs", &fade_in_ms).map_err(|e| format!("Failed to save fade_in_ms: {}", e))?;
        key.set_value("FadeOutMs", &fade_out_ms).map_err(|e| format!("Failed to save fade_out_ms: {}", e))?;
        key.set_value("ChipBg", &chip_bg.to_string()).map_err(|e| format!("Failed to save chip_bg: {}", e))?;
        key.set_value("ChipFg", &chip_fg.to_string()).map_err(|e| format!("Failed to save chip_fg: {}", e))?;
        key.set_value("ChipGap", &chip_gap).map_err(|e| format!("Failed to save chip_gap: {}", e))?;
        key.set_value("ChipPadV", &chip_pad_v).map_err(|e| format!("Failed to save chip_pad_v: {}", e))?;
        key.set_value("ChipPadH", &chip_pad_h).map_err(|e| format!("Failed to save chip_pad_h: {}", e))?;
        key.set_value("ChipRadius", &chip_radius).map_err(|e| format!("Failed to save chip_radius: {}", e))?;
        key.set_value("ChipFontPx", &chip_font_px).map_err(|e| format!("Failed to save chip_font_px: {}", e))?;
        key.set_value("ChipFontWeight", &chip_font_weight).map_err(|e| format!("Failed to save chip_font_weight: {}", e))?;
        key.set_value("Background", &background.to_string()).map_err(|e| format!("Failed to save background: {}", e))?;
        key.set_value("Cols", &cols).map_err(|e| format!("Failed to save cols: {}", e))?;
        key.set_value("Rows", &rows).map_err(|e| format!("Failed to save rows: {}", e))?;
        key.set_value("Align", &align.to_string()).map_err(|e| format!("Failed to save align: {}", e))?;
        key.set_value("Direction", &direction.to_string()).map_err(|e| format!("Failed to save direction: {}", e))?;
        
        return Ok(());
    }
    
    #[cfg(not(target_os = "windows"))]
    Ok(())
}

pub fn load_overlay_config() -> (u32, u32, String, String, u32, u32, u32, u32, u32, u32, String, u32, u32, String, String) {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\KeyViewer\Overlay") {
            let fade_in_ms = key.get_value("FadeInMs").unwrap_or(120);
            let fade_out_ms = key.get_value("FadeOutMs").unwrap_or(120);
            let chip_bg = key.get_value("ChipBg").unwrap_or_else(|_| "rgba(0,0,0,0.6)".to_string());
            let chip_fg = key.get_value("ChipFg").unwrap_or_else(|_| "#ffffff".to_string());
            let chip_gap = key.get_value("ChipGap").unwrap_or(8);
            let chip_pad_v = key.get_value("ChipPadV").unwrap_or(10);
            let chip_pad_h = key.get_value("ChipPadH").unwrap_or(14);
            let chip_radius = key.get_value("ChipRadius").unwrap_or(10);
            let chip_font_px = key.get_value("ChipFontPx").unwrap_or(24);
            let chip_font_weight = key.get_value("ChipFontWeight").unwrap_or(700);
            let background = key.get_value("Background").unwrap_or_else(|_| "rgba(0,0,0,0.0)".to_string());
            let cols = key.get_value("Cols").unwrap_or(8);
            let rows = key.get_value("Rows").unwrap_or(1);
            let align = key.get_value("Align").unwrap_or_else(|_| "center".to_string());
            let direction = key.get_value("Direction").unwrap_or_else(|_| "ltr".to_string());
            
            return (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h, 
                    chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction);
        }
    }
    
    // Default values
    (120, 120, "rgba(0,0,0,0.6)".to_string(), "#ffffff".to_string(), 8, 10, 14, 10, 24, 700, 
     "rgba(0,0,0,0.0)".to_string(), 8, 1, "center".to_string(), "ltr".to_string())
}

/// Reset all settings by deleting the registry key
pub fn reset_all_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = r"Software\KeyViewer";
        
        // Delete the entire KeyViewer registry key
        match hkcu.delete_subkey_all(key_path) {
            Ok(_) => {
                println!("Registry key deleted successfully");
                Ok(())
            },
            Err(e) => {
                // If key doesn't exist, that's also success
                if e.kind() == std::io::ErrorKind::NotFound {
                    println!("Registry key doesn't exist (already reset)");
                    Ok(())
                } else {
                    Err(format!("Failed to delete registry key: {}", e))
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

