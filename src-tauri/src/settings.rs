use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// macOS UserDefaults helper functions
#[cfg(target_os = "macos")]
mod macos_defaults {
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};
    
    // Get NSUserDefaults standardUserDefaults
    unsafe fn get_user_defaults() -> id {
        let class = objc::runtime::Class::get("NSUserDefaults").unwrap();
        msg_send![class, standardUserDefaults]
    }
    
    // Set integer value for key
    pub fn set_integer(key: &str, value: i64) {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let defaults = get_user_defaults();
            let ns_key = NSString::alloc(nil).init_str(key);
            let _: () = msg_send![defaults, setInteger:value forKey:ns_key];
            let _: () = msg_send![defaults, synchronize];
        }
    }
    
    // Get integer value for key with default
    pub fn get_integer(key: &str, default: i64) -> i64 {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let defaults = get_user_defaults();
            let ns_key = NSString::alloc(nil).init_str(key);
            let value: i64 = msg_send![defaults, integerForKey:ns_key];
            if value == 0 {
                // Check if key exists
                let obj: id = msg_send![defaults, objectForKey:ns_key];
                if obj == nil {
                    return default;
                }
            }
            value
        }
    }
    
    // Set string value for key
    pub fn set_string(key: &str, value: &str) {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let defaults = get_user_defaults();
            let ns_key = NSString::alloc(nil).init_str(key);
            let ns_value = NSString::alloc(nil).init_str(value);
            let _: () = msg_send![defaults, setObject:ns_value forKey:ns_key];
            let _: () = msg_send![defaults, synchronize];
        }
    }
    
    // Get string value for key with default
    pub fn get_string(key: &str, default: &str) -> String {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let defaults = get_user_defaults();
            let ns_key = NSString::alloc(nil).init_str(key);
            let ns_value: id = msg_send![defaults, stringForKey:ns_key];
            if ns_value == nil {
                return default.to_string();
            }
            let c_str: *const i8 = msg_send![ns_value, UTF8String];
            if c_str.is_null() {
                return default.to_string();
            }
            std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned()
        }
    }
    
    // Remove value for key
    pub fn remove(key: &str) {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let defaults = get_user_defaults();
            let ns_key = NSString::alloc(nil).init_str(key);
            let _: () = msg_send![defaults, removeObjectForKey:ns_key];
            let _: () = msg_send![defaults, synchronize];
        }
    }
}

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
    // Load settings from Windows Registry or macOS UserDefaults
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
                
                Self {
                    port,
                    language,
                    run_on_startup: run_on_startup != 0,
                }
            } else {
                Self::default()
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let port = macos_defaults::get_integer("com.keyviewer.Port", 8000) as u16;
            let language = macos_defaults::get_string("com.keyviewer.Language", "ko");
            let run_on_startup = macos_defaults::get_integer("com.keyviewer.RunOnStartup", 0) != 0;
            
            Self {
                port,
                language,
                run_on_startup,
            }
        }
        
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            // Linux and other platforms: use default values
            Self::default()
        }
    }

    // Save settings to Windows Registry or macOS UserDefaults
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
        
        #[cfg(target_os = "macos")]
        {
            macos_defaults::set_integer("com.keyviewer.Port", self.port as i64);
            macos_defaults::set_string("com.keyviewer.Language", &self.language);
            macos_defaults::set_integer("com.keyviewer.RunOnStartup", if self.run_on_startup { 1 } else { 0 });
            return Ok(());
        }
        
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            // Linux and other platforms: no-op
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
        // Quote the full path to handle spaces in path
        let exe_str = exe_path.to_string_lossy().to_string();
        let quoted = if exe_str.contains(' ') { format!("\"{}\"", exe_str) } else { exe_str };
        key.set_value("KeyViewer", &quoted)
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

// Save/Load Target Config to/from Registry or UserDefaults
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
    
    #[cfg(target_os = "macos")]
    {
        macos_defaults::set_string("com.keyviewer.TargetMode", mode);
        macos_defaults::set_string("com.keyviewer.TargetValue", value.unwrap_or(""));
        return Ok(());
    }
    
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Linux and other platforms: no-op
        let _ = (mode, value); // Consume to avoid warnings
        Ok(())
    }
}

pub fn load_target_config() -> (String, Option<String>) {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\KeyViewer") {
            let mode: String = key.get_value("TargetMode").unwrap_or_else(|_| "disabled".to_string());
            let value: String = key.get_value("TargetValue").unwrap_or_else(|_| "".to_string());
            let value_opt = if value.is_empty() { None } else { Some(value) };
            (mode, value_opt)
        } else {
            ("disabled".to_string(), None)
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let mode = macos_defaults::get_string("com.keyviewer.TargetMode", "disabled");
        let value = macos_defaults::get_string("com.keyviewer.TargetValue", "");
        let value_opt = if value.is_empty() { None } else { Some(value) };
        (mode, value_opt)
    }
    
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Linux and other platforms: return defaults
        ("disabled".to_string(), None)
    }
}

// Save/Load Overlay Config to/from Registry or UserDefaults
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
    
    #[cfg(target_os = "macos")]
    {
        macos_defaults::set_integer("com.keyviewer.overlay.FadeInMs", fade_in_ms as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.FadeOutMs", fade_out_ms as i64);
        macos_defaults::set_string("com.keyviewer.overlay.ChipBg", chip_bg);
        macos_defaults::set_string("com.keyviewer.overlay.ChipFg", chip_fg);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipGap", chip_gap as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipPadV", chip_pad_v as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipPadH", chip_pad_h as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipRadius", chip_radius as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipFontPx", chip_font_px as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.ChipFontWeight", chip_font_weight as i64);
        macos_defaults::set_string("com.keyviewer.overlay.Background", background);
        macos_defaults::set_integer("com.keyviewer.overlay.Cols", cols as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.Rows", rows as i64);
        macos_defaults::set_string("com.keyviewer.overlay.Align", align);
        macos_defaults::set_string("com.keyviewer.overlay.Direction", direction);
        return Ok(());
    }
    
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Linux and other platforms: no-op
        let _ = (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h, 
                 chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction);
        Ok(())
    }
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
            let background = key.get_value("Background").unwrap_or_else(|_| "rgba(0,0,0,0)".to_string());
            let cols = key.get_value("Cols").unwrap_or(8);
            let rows = key.get_value("Rows").unwrap_or(1);
            let align = key.get_value("Align").unwrap_or_else(|_| "left".to_string());
            let direction = key.get_value("Direction").unwrap_or_else(|_| "ltr".to_string());
            
            (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h, 
             chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction)
        } else {
            // Return default values
            (120, 120, "rgba(0,0,0,0.6)".to_string(), "#ffffff".to_string(), 8, 10, 14, 
             10, 24, 700, "rgba(0,0,0,0)".to_string(), 8, 1, "left".to_string(), "ltr".to_string())
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let fade_in_ms = macos_defaults::get_integer("com.keyviewer.overlay.FadeInMs", 120) as u32;
        let fade_out_ms = macos_defaults::get_integer("com.keyviewer.overlay.FadeOutMs", 120) as u32;
        let chip_bg = macos_defaults::get_string("com.keyviewer.overlay.ChipBg", "rgba(0,0,0,0.6)");
        let chip_fg = macos_defaults::get_string("com.keyviewer.overlay.ChipFg", "#ffffff");
        let chip_gap = macos_defaults::get_integer("com.keyviewer.overlay.ChipGap", 8) as u32;
        let chip_pad_v = macos_defaults::get_integer("com.keyviewer.overlay.ChipPadV", 10) as u32;
        let chip_pad_h = macos_defaults::get_integer("com.keyviewer.overlay.ChipPadH", 14) as u32;
        let chip_radius = macos_defaults::get_integer("com.keyviewer.overlay.ChipRadius", 10) as u32;
        let chip_font_px = macos_defaults::get_integer("com.keyviewer.overlay.ChipFontPx", 24) as u32;
        let chip_font_weight = macos_defaults::get_integer("com.keyviewer.overlay.ChipFontWeight", 700) as u32;
        let background = macos_defaults::get_string("com.keyviewer.overlay.Background", "rgba(0,0,0,0)");
        let cols = macos_defaults::get_integer("com.keyviewer.overlay.Cols", 8) as u32;
        let rows = macos_defaults::get_integer("com.keyviewer.overlay.Rows", 1) as u32;
        let align = macos_defaults::get_string("com.keyviewer.overlay.Align", "left");
        let direction = macos_defaults::get_string("com.keyviewer.overlay.Direction", "ltr");
        
        (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h, 
         chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction)
    }
    
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Linux and other platforms: return defaults
        (120, 120, "rgba(0,0,0,0.6)".to_string(), "#ffffff".to_string(), 8, 10, 14, 10, 24, 700, 
         "rgba(0,0,0,0)".to_string(), 8, 1, "left".to_string(), "ltr".to_string())
    }
}

/// Reset all settings by deleting the registry key or UserDefaults
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
    
    #[cfg(target_os = "macos")]
    {
        // Remove all settings from UserDefaults
        macos_defaults::remove("com.keyviewer.Port");
        macos_defaults::remove("com.keyviewer.Language");
        macos_defaults::remove("com.keyviewer.RunOnStartup");
        macos_defaults::remove("com.keyviewer.TargetMode");
        macos_defaults::remove("com.keyviewer.TargetValue");
        macos_defaults::remove("com.keyviewer.overlay.FadeInMs");
        macos_defaults::remove("com.keyviewer.overlay.FadeOutMs");
        macos_defaults::remove("com.keyviewer.overlay.ChipBg");
        macos_defaults::remove("com.keyviewer.overlay.ChipFg");
        macos_defaults::remove("com.keyviewer.overlay.ChipGap");
        macos_defaults::remove("com.keyviewer.overlay.ChipPadV");
        macos_defaults::remove("com.keyviewer.overlay.ChipPadH");
        macos_defaults::remove("com.keyviewer.overlay.ChipRadius");
        macos_defaults::remove("com.keyviewer.overlay.ChipFontPx");
        macos_defaults::remove("com.keyviewer.overlay.ChipFontWeight");
        macos_defaults::remove("com.keyviewer.overlay.Background");
        macos_defaults::remove("com.keyviewer.overlay.Cols");
        macos_defaults::remove("com.keyviewer.overlay.Rows");
        macos_defaults::remove("com.keyviewer.overlay.Align");
        macos_defaults::remove("com.keyviewer.overlay.Direction");
        println!("UserDefaults settings deleted successfully");
        Ok(())
    }
    
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Linux and other platforms: no-op
        println!("Settings reset (no persistent storage)");
        Ok(())
    }
}

