#![cfg_attr(target_os = "macos", allow(unexpected_cfgs))]

use crate::state::{KeyImagesConfig, KeyStyleConfig};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// macOS UserDefaults helper functions
#[cfg(target_os = "macos")]
mod macos_defaults {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
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

// Linux and other platforms: a flat JSON map on disk. Mirrors the key/value shape
// the Windows registry and macOS UserDefaults paths already use, so every caller
// below stays the same across platforms.
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
mod file_store {
    use serde_json::{Map, Value};
    use std::sync::Mutex;

    // Serializes read-modify-write so two saves cannot lose each other.
    static LOCK: Mutex<()> = Mutex::new(());

    fn path() -> Option<std::path::PathBuf> {
        super::get_config_dir()
            .ok()
            .map(|dir| dir.join("settings.json"))
    }

    fn read() -> Map<String, Value> {
        match path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        {
            Some(Value::Object(map)) => map,
            _ => Map::new(),
        }
    }

    fn write(map: &Map<String, Value>) {
        let Some(path) = path() else {
            eprintln!("[Settings] No config directory available; settings were not saved");
            return;
        };
        match serde_json::to_string_pretty(map) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&path, text) {
                    eprintln!("[Settings] Failed to write {}: {}", path.display(), e);
                }
            }
            Err(e) => eprintln!("[Settings] Failed to serialize settings: {}", e),
        }
    }

    /// Every setting read in one go, so a caller that needs twenty keys touches
    /// the file once instead of twenty times.
    pub struct Snapshot(Map<String, Value>);

    impl Snapshot {
        pub fn integer(&self, key: &str, default: i64) -> i64 {
            self.0.get(key).and_then(Value::as_i64).unwrap_or(default)
        }

        pub fn string(&self, key: &str, default: &str) -> String {
            self.0
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| default.to_string())
        }
    }

    pub fn snapshot() -> Snapshot {
        Snapshot(read())
    }

    /// Collects a group of changes and applies them as a single read-modify-write.
    #[derive(Default)]
    pub struct Batch(Vec<(String, Value)>);

    impl Batch {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn integer(mut self, key: &str, value: i64) -> Self {
            self.0.push((key.to_string(), Value::from(value)));
            self
        }

        pub fn string(mut self, key: &str, value: &str) -> Self {
            self.0.push((key.to_string(), Value::from(value)));
            self
        }

        pub fn commit(self) {
            let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut map = read();
            for (key, value) in self.0 {
                map.insert(key, value);
            }
            write(&map);
        }
    }

    pub fn clear() {
        let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        write(&Map::new());
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
            port: 8000,                 // Default port, always starts with this
            language: "ko".to_string(), // Default language, always starts with Korean
            run_on_startup: false,      // Default disabled, always starts disabled
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
                let language: String = key
                    .get_value("Language")
                    .unwrap_or_else(|_| "ko".to_string());
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
            let store = file_store::snapshot();
            let port = store.integer("Port", 8000).clamp(1, 65535) as u16;
            let language = store.string("Language", "ko");
            let run_on_startup = store.integer("RunOnStartup", 0) != 0;

            Self {
                port,
                language,
                run_on_startup,
            }
        }
    }

    // Save settings to Windows Registry or macOS UserDefaults
    pub fn save(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key_path = r"Software\KeyViewer";

            // Create the key if it doesn't exist
            let (key, _) = hkcu
                .create_subkey(key_path)
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

            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            macos_defaults::set_integer("com.keyviewer.Port", self.port as i64);
            macos_defaults::set_string("com.keyviewer.Language", &self.language);
            macos_defaults::set_integer(
                "com.keyviewer.RunOnStartup",
                if self.run_on_startup { 1 } else { 0 },
            );
            Ok(())
        }

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            file_store::Batch::new()
                .integer("Port", i64::from(self.port))
                .string("Language", &self.language)
                .integer("RunOnStartup", i64::from(self.run_on_startup))
                .commit();
            Ok(())
        }
    }
}

// Windows startup registration
#[cfg(target_os = "windows")]
const STARTUP_TASK_NAME: &str = "KeyQueueViewer";
#[cfg(target_os = "windows")]
const LEGACY_RUN_VALUE_NAME: &str = "KeyViewer";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(target_os = "windows")]
fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str, action: &str) -> Result<(), String> {
    let output = hidden_command("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("Failed to launch PowerShell for {}: {}", action, e))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = format!("{} {}", stderr.trim(), stdout.trim())
        .trim()
        .to_string();
    if details.is_empty() {
        Err(format!("Failed to {}", action))
    } else {
        Err(format!("Failed to {}: {}", action, details))
    }
}

#[cfg(target_os = "windows")]
fn remove_legacy_run_value() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
        let _ = key.delete_value(LEGACY_RUN_VALUE_NAME);
    }
}

#[cfg(target_os = "windows")]
pub fn set_startup_registration(enabled: bool) -> Result<(), String> {
    if enabled && cfg!(debug_assertions) {
        remove_legacy_run_value();
        return Err(
            "Refusing to register a debug build for Windows startup. Use a release executable."
                .to_string(),
        );
    }

    if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
        let working_dir = exe_path
            .parent()
            .ok_or_else(|| "Failed to get exe directory".to_string())?;
        let exe_path = powershell_quote(&exe_path.to_string_lossy());
        let working_dir = powershell_quote(&working_dir.to_string_lossy());
        let task_name = powershell_quote(STARTUP_TASK_NAME);

        let script = format!(
            r#"
$ErrorActionPreference = 'Stop'
$user = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
$action = New-ScheduledTaskAction -Execute {exe_path} -WorkingDirectory {working_dir}
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $user
$principal = New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Highest
$task = New-ScheduledTask -Action $action -Trigger $trigger -Principal $principal
Register-ScheduledTask -TaskName {task_name} -InputObject $task -Force | Out-Null
"#
        );

        run_powershell(&script, "register Windows startup task")?;
    } else {
        let task_name = powershell_quote(STARTUP_TASK_NAME);
        let script = format!(
            r#"
Unregister-ScheduledTask -TaskName {task_name} -Confirm:$false -ErrorAction SilentlyContinue
"#
        );

        run_powershell(&script, "unregister Windows startup task")?;
    }

    remove_legacy_run_value();
    Ok(())
}

// macOS: a per-user LaunchAgent. Writing the plist is enough for the next login;
// launchctl is called so the change also takes effect in the current session.
#[cfg(target_os = "macos")]
const LAUNCH_AGENT_LABEL: &str = "com.keyviewer.launcher";

#[cfg(target_os = "macos")]
fn launch_agent_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or_else(|| "HOME not found".to_string())?;
    let dir = std::path::PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;
    Ok(dir.join(format!("{}.plist", LAUNCH_AGENT_LABEL)))
}

#[cfg(target_os = "macos")]
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(target_os = "macos")]
pub fn set_startup_registration(enabled: bool) -> Result<(), String> {
    if enabled && cfg!(debug_assertions) {
        return Err(
            "Refusing to register a debug build for login startup. Use a release executable."
                .to_string(),
        );
    }

    let plist_path = launch_agent_path()?;
    let plist_arg = plist_path.to_string_lossy().to_string();

    if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
            label = LAUNCH_AGENT_LABEL,
            exe = xml_escape(&exe_path.to_string_lossy()),
        );

        std::fs::write(&plist_path, plist)
            .map_err(|e| format!("Failed to write {}: {}", plist_path.display(), e))?;

        // Reload so the agent is registered without waiting for the next login.
        let _ = Command::new("launchctl")
            .args(["unload", &plist_arg])
            .output();
        Command::new("launchctl")
            .args(["load", "-w", &plist_arg])
            .output()
            .map_err(|e| format!("Failed to run launchctl: {}", e))?;
    } else {
        let _ = Command::new("launchctl")
            .args(["unload", "-w", &plist_arg])
            .output();
        if plist_path.exists() {
            std::fs::remove_file(&plist_path)
                .map_err(|e| format!("Failed to remove {}: {}", plist_path.display(), e))?;
        }
    }

    Ok(())
}

// Linux and other platforms: an XDG autostart desktop entry.
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn autostart_dir() -> Result<std::path::PathBuf, String> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return Ok(std::path::PathBuf::from(config_home).join("autostart"));
        }
    }
    let home = std::env::var_os("HOME").ok_or_else(|| "HOME not found".to_string())?;
    Ok(std::path::PathBuf::from(home)
        .join(".config")
        .join("autostart"))
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn set_startup_registration(enabled: bool) -> Result<(), String> {
    if enabled && cfg!(debug_assertions) {
        return Err(
            "Refusing to register a debug build for login startup. Use a release executable."
                .to_string(),
        );
    }

    let entry = autostart_dir()?.join("keyviewer.desktop");

    if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
        // Exec is a quoted string per the Desktop Entry spec, so paths with spaces work.
        let exec = exe_path
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let desktop = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=KeyQueueViewer\n\
             Exec=\"{exec}\"\n\
             Terminal=false\n\
             X-GNOME-Autostart-enabled=true\n"
        );

        if let Some(dir) = entry.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;
        }
        std::fs::write(&entry, desktop)
            .map_err(|e| format!("Failed to write {}: {}", entry.display(), e))?;
    } else if entry.exists() {
        std::fs::remove_file(&entry)
            .map_err(|e| format!("Failed to remove {}: {}", entry.display(), e))?;
    }

    Ok(())
}

// Save/Load Target Config to/from Registry or UserDefaults
pub fn save_target_config(mode: &str, value: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(r"Software\KeyViewer")
            .map_err(|e| format!("Failed to create registry key: {}", e))?;

        key.set_value("TargetMode", &mode.to_string())
            .map_err(|e| format!("Failed to save target mode: {}", e))?;
        key.set_value("TargetValue", &value.unwrap_or(""))
            .map_err(|e| format!("Failed to save target value: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        macos_defaults::set_string("com.keyviewer.TargetMode", mode);
        macos_defaults::set_string("com.keyviewer.TargetValue", value.unwrap_or(""));
        Ok(())
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        file_store::Batch::new()
            .string("TargetMode", mode)
            .string("TargetValue", value.unwrap_or(""))
            .commit();
        Ok(())
    }
}

pub fn load_target_config() -> (String, Option<String>) {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\KeyViewer") {
            let mode: String = key
                .get_value("TargetMode")
                .unwrap_or_else(|_| "disabled".to_string());
            let value: String = key
                .get_value("TargetValue")
                .unwrap_or_else(|_| "".to_string());
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
        let store = file_store::snapshot();
        let mode = store.string("TargetMode", "disabled");
        let value = store.string("TargetValue", "");
        let value_opt = if value.is_empty() { None } else { Some(value) };
        (mode, value_opt)
    }
}

// Save/Load Overlay Config to/from Registry or UserDefaults
#[allow(clippy::too_many_arguments)]
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
    color_mode: &str,
    grad_color1: &str,
    grad_color2: &str,
    grad_dir: &str,
    overlay_mode: &str,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(r"Software\KeyViewer\Overlay")
            .map_err(|e| format!("Failed to create registry key: {}", e))?;

        // Save all overlay settings
        key.set_value("FadeInMs", &fade_in_ms)
            .map_err(|e| format!("Failed to save fade_in_ms: {}", e))?;
        key.set_value("FadeOutMs", &fade_out_ms)
            .map_err(|e| format!("Failed to save fade_out_ms: {}", e))?;
        key.set_value("ChipBg", &chip_bg.to_string())
            .map_err(|e| format!("Failed to save chip_bg: {}", e))?;
        key.set_value("ChipFg", &chip_fg.to_string())
            .map_err(|e| format!("Failed to save chip_fg: {}", e))?;
        key.set_value("ChipGap", &chip_gap)
            .map_err(|e| format!("Failed to save chip_gap: {}", e))?;
        key.set_value("ChipPadV", &chip_pad_v)
            .map_err(|e| format!("Failed to save chip_pad_v: {}", e))?;
        key.set_value("ChipPadH", &chip_pad_h)
            .map_err(|e| format!("Failed to save chip_pad_h: {}", e))?;
        key.set_value("ChipRadius", &chip_radius)
            .map_err(|e| format!("Failed to save chip_radius: {}", e))?;
        key.set_value("ChipFontPx", &chip_font_px)
            .map_err(|e| format!("Failed to save chip_font_px: {}", e))?;
        key.set_value("ChipFontWeight", &chip_font_weight)
            .map_err(|e| format!("Failed to save chip_font_weight: {}", e))?;
        key.set_value("Background", &background.to_string())
            .map_err(|e| format!("Failed to save background: {}", e))?;
        key.set_value("Cols", &cols)
            .map_err(|e| format!("Failed to save cols: {}", e))?;
        key.set_value("Rows", &rows)
            .map_err(|e| format!("Failed to save rows: {}", e))?;
        key.set_value("Align", &align.to_string())
            .map_err(|e| format!("Failed to save align: {}", e))?;
        key.set_value("Direction", &direction.to_string())
            .map_err(|e| format!("Failed to save direction: {}", e))?;
        key.set_value("ColorMode", &color_mode.to_string())
            .map_err(|e| format!("Failed to save color_mode: {}", e))?;
        key.set_value("GradColor1", &grad_color1.to_string())
            .map_err(|e| format!("Failed to save grad_color1: {}", e))?;
        key.set_value("GradColor2", &grad_color2.to_string())
            .map_err(|e| format!("Failed to save grad_color2: {}", e))?;
        key.set_value("GradDir", &grad_dir.to_string())
            .map_err(|e| format!("Failed to save grad_dir: {}", e))?;
        key.set_value("OverlayMode", &overlay_mode.to_string())
            .map_err(|e| format!("Failed to save overlay_mode: {}", e))?;

        Ok(())
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
        macos_defaults::set_integer(
            "com.keyviewer.overlay.ChipFontWeight",
            chip_font_weight as i64,
        );
        macos_defaults::set_string("com.keyviewer.overlay.Background", background);
        macos_defaults::set_integer("com.keyviewer.overlay.Cols", cols as i64);
        macos_defaults::set_integer("com.keyviewer.overlay.Rows", rows as i64);
        macos_defaults::set_string("com.keyviewer.overlay.Align", align);
        macos_defaults::set_string("com.keyviewer.overlay.Direction", direction);
        macos_defaults::set_string("com.keyviewer.overlay.ColorMode", color_mode);
        macos_defaults::set_string("com.keyviewer.overlay.GradColor1", grad_color1);
        macos_defaults::set_string("com.keyviewer.overlay.GradColor2", grad_color2);
        macos_defaults::set_string("com.keyviewer.overlay.GradDir", grad_dir);
        macos_defaults::set_string("com.keyviewer.overlay.OverlayMode", overlay_mode);
        Ok(())
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        file_store::Batch::new()
            .integer("overlay.FadeInMs", i64::from(fade_in_ms))
            .integer("overlay.FadeOutMs", i64::from(fade_out_ms))
            .string("overlay.ChipBg", chip_bg)
            .string("overlay.ChipFg", chip_fg)
            .integer("overlay.ChipGap", i64::from(chip_gap))
            .integer("overlay.ChipPadV", i64::from(chip_pad_v))
            .integer("overlay.ChipPadH", i64::from(chip_pad_h))
            .integer("overlay.ChipRadius", i64::from(chip_radius))
            .integer("overlay.ChipFontPx", i64::from(chip_font_px))
            .integer("overlay.ChipFontWeight", i64::from(chip_font_weight))
            .string("overlay.Background", background)
            .integer("overlay.Cols", i64::from(cols))
            .integer("overlay.Rows", i64::from(rows))
            .string("overlay.Align", align)
            .string("overlay.Direction", direction)
            .string("overlay.ColorMode", color_mode)
            .string("overlay.GradColor1", grad_color1)
            .string("overlay.GradColor2", grad_color2)
            .string("overlay.GradDir", grad_dir)
            .string("overlay.OverlayMode", overlay_mode)
            .commit();
        Ok(())
    }
}

// Returns: (fade_in_ms, fade_out_ms, chip_bg, chip_fg, chip_gap, chip_pad_v, chip_pad_h,
//           chip_radius, chip_font_px, chip_font_weight, background, cols, rows, align, direction,
//           color_mode, grad_color1, grad_color2, grad_dir, overlay_mode)
#[allow(clippy::type_complexity)]
pub fn load_overlay_config() -> (
    u32,
    u32,
    String,
    String,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    String,
    u32,
    u32,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Software\KeyViewer\Overlay") {
            let fade_in_ms = key.get_value("FadeInMs").unwrap_or(120);
            let fade_out_ms = key.get_value("FadeOutMs").unwrap_or(120);
            let chip_bg = key
                .get_value("ChipBg")
                .unwrap_or_else(|_| "#000000".to_string());
            let chip_fg = key
                .get_value("ChipFg")
                .unwrap_or_else(|_| "#ffffff".to_string());
            let chip_gap = key.get_value("ChipGap").unwrap_or(8);
            let chip_pad_v = key.get_value("ChipPadV").unwrap_or(10);
            let chip_pad_h = key.get_value("ChipPadH").unwrap_or(14);
            let chip_radius = key.get_value("ChipRadius").unwrap_or(10);
            let chip_font_px = key.get_value("ChipFontPx").unwrap_or(24);
            let chip_font_weight = key.get_value("ChipFontWeight").unwrap_or(700);
            let background = key
                .get_value("Background")
                .unwrap_or_else(|_| "rgba(0,0,0,0)".to_string());
            let cols = key.get_value("Cols").unwrap_or(8);
            let rows = key.get_value("Rows").unwrap_or(1);
            let align = key
                .get_value("Align")
                .unwrap_or_else(|_| "left".to_string());
            let direction = key
                .get_value("Direction")
                .unwrap_or_else(|_| "ltr".to_string());
            let color_mode = key
                .get_value("ColorMode")
                .unwrap_or_else(|_| "solid".to_string());
            let grad_color1 = key
                .get_value("GradColor1")
                .unwrap_or_else(|_| "#000000".to_string());
            let grad_color2 = key
                .get_value("GradColor2")
                .unwrap_or_else(|_| "#333333".to_string());
            let grad_dir = key
                .get_value("GradDir")
                .unwrap_or_else(|_| "to bottom".to_string());
            let overlay_mode = key
                .get_value("OverlayMode")
                .unwrap_or_else(|_| "queue".to_string());

            (
                fade_in_ms,
                fade_out_ms,
                chip_bg,
                chip_fg,
                chip_gap,
                chip_pad_v,
                chip_pad_h,
                chip_radius,
                chip_font_px,
                chip_font_weight,
                background,
                cols,
                rows,
                align,
                direction,
                color_mode,
                grad_color1,
                grad_color2,
                grad_dir,
                overlay_mode,
            )
        } else {
            // Return default values
            (
                120,
                120,
                "#000000".to_string(),
                "#ffffff".to_string(),
                8,
                10,
                14,
                10,
                24,
                700,
                "rgba(0,0,0,0)".to_string(),
                8,
                1,
                "left".to_string(),
                "ltr".to_string(),
                "solid".to_string(),
                "#000000".to_string(),
                "#333333".to_string(),
                "to bottom".to_string(),
                "queue".to_string(),
            )
        }
    }

    #[cfg(target_os = "macos")]
    {
        let fade_in_ms = macos_defaults::get_integer("com.keyviewer.overlay.FadeInMs", 120) as u32;
        let fade_out_ms =
            macos_defaults::get_integer("com.keyviewer.overlay.FadeOutMs", 120) as u32;
        let chip_bg = macos_defaults::get_string("com.keyviewer.overlay.ChipBg", "#000000");
        let chip_fg = macos_defaults::get_string("com.keyviewer.overlay.ChipFg", "#ffffff");
        let chip_gap = macos_defaults::get_integer("com.keyviewer.overlay.ChipGap", 8) as u32;
        let chip_pad_v = macos_defaults::get_integer("com.keyviewer.overlay.ChipPadV", 10) as u32;
        let chip_pad_h = macos_defaults::get_integer("com.keyviewer.overlay.ChipPadH", 14) as u32;
        let chip_radius =
            macos_defaults::get_integer("com.keyviewer.overlay.ChipRadius", 10) as u32;
        let chip_font_px =
            macos_defaults::get_integer("com.keyviewer.overlay.ChipFontPx", 24) as u32;
        let chip_font_weight =
            macos_defaults::get_integer("com.keyviewer.overlay.ChipFontWeight", 700) as u32;
        let background =
            macos_defaults::get_string("com.keyviewer.overlay.Background", "rgba(0,0,0,0)");
        let cols = macos_defaults::get_integer("com.keyviewer.overlay.Cols", 8) as u32;
        let rows = macos_defaults::get_integer("com.keyviewer.overlay.Rows", 1) as u32;
        let align = macos_defaults::get_string("com.keyviewer.overlay.Align", "left");
        let direction = macos_defaults::get_string("com.keyviewer.overlay.Direction", "ltr");
        let color_mode = macos_defaults::get_string("com.keyviewer.overlay.ColorMode", "solid");
        let grad_color1 = macos_defaults::get_string("com.keyviewer.overlay.GradColor1", "#000000");
        let grad_color2 = macos_defaults::get_string("com.keyviewer.overlay.GradColor2", "#333333");
        let grad_dir = macos_defaults::get_string("com.keyviewer.overlay.GradDir", "to bottom");
        let overlay_mode = macos_defaults::get_string("com.keyviewer.overlay.OverlayMode", "queue");

        (
            fade_in_ms,
            fade_out_ms,
            chip_bg,
            chip_fg,
            chip_gap,
            chip_pad_v,
            chip_pad_h,
            chip_radius,
            chip_font_px,
            chip_font_weight,
            background,
            cols,
            rows,
            align,
            direction,
            color_mode,
            grad_color1,
            grad_color2,
            grad_dir,
            overlay_mode,
        )
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let store = file_store::snapshot();
        let fade_in_ms = store
            .integer("overlay.FadeInMs", 120)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let fade_out_ms = store
            .integer("overlay.FadeOutMs", 120)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_bg = store.string("overlay.ChipBg", "#000000");
        let chip_fg = store.string("overlay.ChipFg", "#ffffff");
        let chip_gap = store
            .integer("overlay.ChipGap", 8)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_pad_v = store
            .integer("overlay.ChipPadV", 10)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_pad_h = store
            .integer("overlay.ChipPadH", 14)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_radius = store
            .integer("overlay.ChipRadius", 10)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_font_px = store
            .integer("overlay.ChipFontPx", 24)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let chip_font_weight = store
            .integer("overlay.ChipFontWeight", 700)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let background = store.string("overlay.Background", "rgba(0,0,0,0)");
        let cols = store
            .integer("overlay.Cols", 8)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let rows = store
            .integer("overlay.Rows", 1)
            .clamp(0, i64::from(u32::MAX)) as u32;
        let align = store.string("overlay.Align", "left");
        let direction = store.string("overlay.Direction", "ltr");
        let color_mode = store.string("overlay.ColorMode", "solid");
        let grad_color1 = store.string("overlay.GradColor1", "#000000");
        let grad_color2 = store.string("overlay.GradColor2", "#333333");
        let grad_dir = store.string("overlay.GradDir", "to bottom");
        let overlay_mode = store.string("overlay.OverlayMode", "queue");

        (
            fade_in_ms,
            fade_out_ms,
            chip_bg,
            chip_fg,
            chip_gap,
            chip_pad_v,
            chip_pad_h,
            chip_radius,
            chip_font_px,
            chip_font_weight,
            background,
            cols,
            rows,
            align,
            direction,
            color_mode,
            grad_color1,
            grad_color2,
            grad_dir,
            overlay_mode,
        )
    }
}

// Save/Load Key Images Config to/from file (JSON)
// Using file-based storage since key images can be large (base64)
pub fn save_key_images_config(config: &KeyImagesConfig) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let file_path = config_dir.join("key_images.json");

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize key images: {}", e))?;

    std::fs::write(&file_path, json)
        .map_err(|e| format!("Failed to write key images file: {}", e))?;

    Ok(())
}

pub fn load_key_images_config() -> KeyImagesConfig {
    if let Ok(config_dir) = get_config_dir() {
        let file_path = config_dir.join("key_images.json");
        if let Ok(json) = std::fs::read_to_string(&file_path) {
            if let Ok(config) = serde_json::from_str(&json) {
                return config;
            }
        }
    }
    KeyImagesConfig::default()
}

// Save/Load Key Style Config to/from file (JSON)
pub fn save_key_style_config(config: &KeyStyleConfig) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let file_path = config_dir.join("key_style.json");

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize key style: {}", e))?;

    std::fs::write(&file_path, json)
        .map_err(|e| format!("Failed to write key style file: {}", e))?;

    Ok(())
}

pub fn load_key_style_config() -> KeyStyleConfig {
    if let Ok(config_dir) = get_config_dir() {
        let file_path = config_dir.join("key_style.json");
        if let Ok(json) = std::fs::read_to_string(&file_path) {
            if let Ok(config) = serde_json::from_str(&json) {
                return config;
            }
        }
    }
    KeyStyleConfig::default()
}

fn get_config_dir() -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            let path = std::path::PathBuf::from(app_data).join("KeyViewer");
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            return Ok(path);
        }
        Err("APPDATA not found".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("KeyViewer");
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            return Ok(path);
        }
        Err("HOME not found".to_string())
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home)
                .join(".config")
                .join("keyviewer");
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            return Ok(path);
        }
        Err("HOME not found".to_string())
    }
}

/// Reset all settings by deleting the registry key or UserDefaults
pub fn reset_all_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = set_startup_registration(false) {
            eprintln!("Warning: failed to remove Windows startup task: {}", e);
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = r"Software\KeyViewer";

        // Delete the entire KeyViewer registry key
        match hkcu.delete_subkey_all(key_path) {
            Ok(_) => {
                println!("Registry key deleted successfully");
                Ok(())
            }
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
        if let Err(e) = set_startup_registration(false) {
            eprintln!("Warning: failed to remove the login startup agent: {}", e);
        }

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
        macos_defaults::remove("com.keyviewer.overlay.ColorMode");
        macos_defaults::remove("com.keyviewer.overlay.GradColor1");
        macos_defaults::remove("com.keyviewer.overlay.GradColor2");
        macos_defaults::remove("com.keyviewer.overlay.GradDir");
        macos_defaults::remove("com.keyviewer.overlay.OverlayMode");
        println!("UserDefaults settings deleted successfully");
        Ok(())
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Err(e) = set_startup_registration(false) {
            eprintln!("Warning: failed to remove autostart entry: {}", e);
        }
        file_store::clear();
        println!("Settings file cleared successfully");
        Ok(())
    }
}
