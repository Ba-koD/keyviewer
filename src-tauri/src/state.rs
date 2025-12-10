use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

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
    // Gradient settings
    #[serde(default = "default_color_mode")]
    pub color_mode: String,
    #[serde(default = "default_grad_color1")]
    pub grad_color1: String,
    #[serde(default = "default_grad_color2")]
    pub grad_color2: String,
    #[serde(default = "default_grad_dir")]
    pub grad_dir: String,
}

fn default_color_mode() -> String {
    "solid".to_string()
}
fn default_grad_color1() -> String {
    "#000000".to_string()
}
fn default_grad_color2() -> String {
    "#333333".to_string()
}
fn default_grad_dir() -> String {
    "to bottom".to_string()
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            fade_in_ms: 120,
            fade_out_ms: 120,
            chip_bg: "#000000".to_string(),
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
            color_mode: "solid".to_string(),
            grad_color1: "#000000".to_string(),
            grad_color2: "#333333".to_string(),
            grad_dir: "to bottom".to_string(),
        }
    }
}

// ============= New Style System =============

/// Background configuration (for overlay background or chip background)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    pub transparent: bool,
    pub mode: String, // "solid" | "gradient" | "image"
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub gradient: Option<GradientConfig>,
    #[serde(default)]
    pub image: Option<String>, // Base64 encoded
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            transparent: true,
            mode: "solid".to_string(),
            color: Some("#000000".to_string()),
            gradient: None,
            image: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientConfig {
    pub color1: String,
    pub color2: String,
    pub direction: String, // "to bottom", "to right", etc.
}

impl Default for GradientConfig {
    fn default() -> Self {
        Self {
            color1: "#000000".to_string(),
            color2: "#333333".to_string(),
            direction: "to bottom".to_string(),
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_font_family")]
    pub family: String,
    #[serde(default = "default_font_url")]
    pub url: Option<String>, // Google Fonts URL or custom font URL
    #[serde(default = "default_font_size")]
    pub size: u32,
    #[serde(default = "default_font_weight")]
    pub weight: u32,
    #[serde(rename = "colorMode", default = "default_color_mode_str")]
    pub color_mode: String, // "solid" | "gradient"
    #[serde(default = "default_font_color")]
    pub color: String,
    #[serde(default)]
    pub gradient: Option<GradientConfig>,
    #[serde(default)]
    pub transparent: bool, // Hide text
    #[serde(default = "default_shadow")]
    pub shadow: bool,
}

fn default_font_family() -> String {
    "system-ui".to_string()
}
fn default_font_url() -> Option<String> {
    None
}
fn default_font_size() -> u32 {
    24
}
fn default_font_weight() -> u32 {
    700
}
fn default_color_mode_str() -> String {
    "solid".to_string()
}
fn default_font_color() -> String {
    "#ffffff".to_string()
}
fn default_shadow() -> bool {
    true
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "system-ui".to_string(),
            url: None,
            size: 24,
            weight: 700,
            color_mode: "solid".to_string(),
            color: "#ffffff".to_string(),
            gradient: None,
            transparent: false,
            shadow: true,
        }
    }
}

/// A style group - can be applied to specific keys, group of keys, or all keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleGroup {
    pub id: String,
    pub name: String,
    #[serde(rename = "groupType")]
    pub group_type: String, // "individual" | "group" | "all"
    pub keys: Vec<String>, // Empty for "all" type
    #[serde(rename = "chipBg")]
    pub chip_bg: BackgroundConfig,
    pub font: FontConfig,
}

/// Main style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStyleConfig {
    pub background: BackgroundConfig, // Overall overlay background
    #[serde(rename = "styleGroups")]
    pub style_groups: Vec<StyleGroup>,
    #[serde(rename = "defaultFont")]
    pub default_font: FontConfig,
    // Chip layout settings
    #[serde(rename = "chipGap", default = "default_chip_gap")]
    pub chip_gap: u32,
    #[serde(rename = "chipPadV", default = "default_chip_pad_v")]
    pub chip_pad_v: u32,
    #[serde(rename = "chipPadH", default = "default_chip_pad_h")]
    pub chip_pad_h: u32,
    #[serde(rename = "chipRadius", default = "default_chip_radius")]
    pub chip_radius: u32,
}

fn default_chip_gap() -> u32 {
    8
}
fn default_chip_pad_v() -> u32 {
    10
}
fn default_chip_pad_h() -> u32 {
    14
}
fn default_chip_radius() -> u32 {
    10
}

impl Default for KeyStyleConfig {
    fn default() -> Self {
        Self {
            background: BackgroundConfig::default(),
            style_groups: Vec::new(),
            default_font: FontConfig::default(),
            chip_gap: 8,
            chip_pad_v: 10,
            chip_pad_h: 14,
            chip_radius: 10,
        }
    }
}

// ============= Legacy structures (keep for backward compatibility) =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImageData {
    pub image: String, // Base64 encoded image
    #[serde(rename = "textColor", default = "default_text_color")]
    pub text_color: Option<String>,
    #[serde(rename = "textOpacity", default)]
    pub text_opacity: Option<f64>,
}

fn default_text_color() -> Option<String> {
    Some("#ffffff".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGroupData {
    pub keys: Vec<String>,
    pub image: String, // Base64 encoded image
    #[serde(rename = "keyType", default)]
    pub key_type: Option<String>,
    #[serde(rename = "textColor", default)]
    pub text_color: Option<String>,
    #[serde(rename = "textOpacity", default)]
    pub text_opacity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyImagesConfig {
    pub individual: std::collections::HashMap<String, KeyImageData>,
    pub groups: Vec<KeyGroupData>,
    #[serde(rename = "allKeys")]
    pub all_keys: Option<KeyImageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub overlay: OverlayConfig,
    pub key_images: KeyImagesConfig,
    #[serde(rename = "keyStyle", default)]
    pub key_style: KeyStyleConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            overlay: OverlayConfig::default(),
            key_images: KeyImagesConfig::default(),
            key_style: KeyStyleConfig::default(),
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
    #[allow(dead_code)]
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.key_labels.contains_key(&key_code)
    }

    pub fn bump_cache_buster(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        self.cache_buster = now.as_millis() as u64;
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.label_order.iter().cloned().collect()
    }
}
