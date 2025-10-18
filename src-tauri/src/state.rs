use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::sync::watch;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub mode: String, // "disabled" | "title" | "process" | "hwnd" | "class" | "all"
    pub value: Option<String>,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            mode: "disabled".to_string(),
            value: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub fade_in_ms: u32,
    pub fade_out_ms: u32,
    pub chip_bg: String,
    pub chip_fg: String,
    pub chip_gap: u32,
    pub chip_pad_v: u32,
    pub chip_pad_h: u32,
    pub chip_radius: u32,
    pub chip_font_px: u32,
    pub chip_font_weight: u32,
    pub background: String,
    pub cols: u32,
    pub rows: u32,
    pub single_line: bool,
    pub single_line_scale: u32,
    pub align: String,
    pub direction: String,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            fade_in_ms: 120,
            fade_out_ms: 120,
            chip_bg: "rgba(0,0,0,0.6)".to_string(),
            chip_fg: "#ffffff".to_string(),
            chip_gap: 8,
            chip_pad_v: 10,
            chip_pad_h: 14,
            chip_radius: 10,
            chip_font_px: 24,
            chip_font_weight: 700,
            background: "rgba(0,0,0,0.0)".to_string(),
            cols: 8,
            rows: 1,
            single_line: false,
            single_line_scale: 90,
            align: "center".to_string(),
            direction: "ltr".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub overlay: OverlayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            overlay: OverlayConfig::default(),
        }
    }
}

pub struct AppState {
    // Pressed keys in order
    pub pressed_keys: VecDeque<String>,
    // Map of key code to label for tracking
    pub key_labels: HashMap<u32, String>,
    // Target window configuration
    pub target_config: TargetConfig,
    // Application configuration
    pub app_config: AppConfig,
    // Language setting
    pub language: String,
    // Server alive flag to terminate active websocket loops on stop
    pub server_alive: bool,
    // Outgoing key updates for immediate WS pushes
    pub event_tx: Option<watch::Sender<Vec<String>>>,
    // Cache buster to invalidate OBS/browser cache on start/config change
    pub cache_buster: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pressed_keys: VecDeque::new(),
            key_labels: HashMap::new(),
            target_config: TargetConfig::default(),
            app_config: AppConfig::default(),
            language: "ko".to_string(), // 기본값을 한국어로
            server_alive: false,
            event_tx: None,
            cache_buster: 0,
        }
    }

    pub fn set_event_tx(&mut self, tx: watch::Sender<Vec<String>>) {
        self.event_tx = Some(tx);
    }

    pub fn add_key(&mut self, key_code: u32, label: String) {
        if !self.key_labels.contains_key(&key_code) {
            self.key_labels.insert(key_code, label.clone());
            self.pressed_keys.push_back(label);
            if let Some(tx) = &self.event_tx { let _ = tx.send(self.get_keys()); }
        }
    }

    pub fn remove_key(&mut self, key_code: u32) {
        if let Some(label) = self.key_labels.remove(&key_code) {
            self.pressed_keys.retain(|k| k != &label);
            if let Some(tx) = &self.event_tx { let _ = tx.send(self.get_keys()); }
        }
    }

    pub fn clear_keys(&mut self) {
        self.pressed_keys.clear();
        self.key_labels.clear();
        if let Some(tx) = &self.event_tx { let _ = tx.send(self.get_keys()); }
    }

    pub fn bump_cache_buster(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        self.cache_buster = now.as_millis() as u64;
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.pressed_keys.iter().cloned().collect()
    }
}

