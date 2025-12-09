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
    // Key image settings (stored as JSON string, parsed in JS)
    // Format: {"default": "data:image/...", "S": "data:image/...", ...}
    #[serde(default)]
    pub key_images: String,
    // Hide text settings (stored as JSON string)
    // Format: {"default": false, "S": true, ...}
    #[serde(default)]
    pub hide_key_text: String,
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
            background: "rgba(0,0,0,0)".to_string(), // Transparent background by default
            cols: 8,
            rows: 1,
            single_line: false,
            single_line_scale: 90,
            align: "left".to_string(), // Left alignment by default
            direction: "ltr".to_string(),
            key_images: "{}".to_string(), // Empty JSON object
            hide_key_text: "{}".to_string(), // Empty JSON object
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
    // Map of key code to label for tracking (code -> label)
    pub key_labels: HashMap<u32, String>,
    // Reference count for each label (multiple keys can have same label, e.g. ShiftLeft/ShiftRight both = "SHIFT")
    pub label_counts: HashMap<String, u32>,
    // Order of first press for each unique label (for display order)
    pub label_order: VecDeque<String>,
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
            key_labels: HashMap::new(),
            label_counts: HashMap::new(),
            label_order: VecDeque::new(),
            target_config: TargetConfig::default(),
            app_config: AppConfig::default(),
            language: "ko".to_string(), // Default to Korean
            server_alive: false,
            event_tx: None,
            cache_buster: 0,
        }
    }

    pub fn set_event_tx(&mut self, tx: watch::Sender<Vec<String>>) {
        self.event_tx = Some(tx);
    }

    pub fn add_key(&mut self, key_code: u32, label: String) {
        // Skip if this exact key code is already tracked
        if self.key_labels.contains_key(&key_code) {
            return;
        }
        
        // Track this key code -> label mapping
        self.key_labels.insert(key_code, label.clone());
        
        // Increment reference count for this label
        let count = self.label_counts.entry(label.clone()).or_insert(0);
        *count += 1;
        
        // Only add to display order if this is the first key with this label
        if *count == 1 {
            self.label_order.push_back(label);
        }
        
        if let Some(tx) = &self.event_tx { 
            let _ = tx.send(self.get_keys()); 
        }
    }

    pub fn remove_key(&mut self, key_code: u32) {
        // Get and remove the label for this key code
        if let Some(label) = self.key_labels.remove(&key_code) {
            // Decrement reference count
            if let Some(count) = self.label_counts.get_mut(&label) {
                *count = count.saturating_sub(1);
                
                // Only remove from display when NO keys with this label are pressed
                if *count == 0 {
                    self.label_counts.remove(&label);
                    self.label_order.retain(|l| l != &label);
                }
            }
            
            if let Some(tx) = &self.event_tx { 
                let _ = tx.send(self.get_keys()); 
            }
        }
    }

    pub fn clear_keys(&mut self) {
        self.key_labels.clear();
        self.label_counts.clear();
        self.label_order.clear();
        if let Some(tx) = &self.event_tx { 
            let _ = tx.send(self.get_keys()); 
        }
    }

    // Check if a key code is currently tracked
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.key_labels.contains_key(&key_code)
    }

    pub fn bump_cache_buster(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        self.cache_buster = now.as_millis() as u64;
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.label_order.iter().cloned().collect()
    }
}

