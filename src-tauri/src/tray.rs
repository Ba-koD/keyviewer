// Tray icon that mirrors the server state.
//
// While the HTTP server is up the icon carries a red dot - the same "we are live"
// cue OBS uses for an active recording - so the state stays readable at a glance
// even when the window is hidden. On Windows the taskbar button gets the same dot
// as an overlay icon, the way Discord marks unread activity.

use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::settings::LauncherSettings;
use crate::AppHandle as LauncherState;

const TRAY_ID: &str = "main-tray";

// The bundled icon carries every size from 16 to 128, so the tray can be fed a
// source that matches the shell DPI instead of an upscaled 16px entry.
const ICON_ICO: &[u8] = include_bytes!("../icons/icon.ico");

const DOT_RGB: [u8; 3] = [0xF2, 0x3F, 0x42];
#[cfg(target_os = "windows")]
const RING_RGB: [u8; 3] = [0xFF, 0xFF, 0xFF];

struct TrayItems {
    status: MenuItem<Wry>,
    start: MenuItem<Wry>,
    stop: MenuItem<Wry>,
}

static ITEMS: OnceLock<TrayItems> = OnceLock::new();
static ICON_IDLE: OnceLock<(Vec<u8>, u32)> = OnceLock::new();
static ICON_LIVE: OnceLock<(Vec<u8>, u32)> = OnceLock::new();
#[cfg(target_os = "windows")]
static OVERLAY_LIVE: OnceLock<(Vec<u8>, u32)> = OnceLock::new();

/// Builds the tray icon and keeps it in sync with `running` for the rest of the
/// process lifetime.
pub fn init(app: &AppHandle<Wry>, running: Arc<Mutex<bool>>) {
    let live = *running.lock();

    if let Err(err) = create(app, live) {
        eprintln!("[Tray] Failed to create tray icon: {}", err);
        return;
    }

    spawn_watcher(app.clone(), running, live);

    #[cfg(target_os = "windows")]
    spawn_promotion();
}

fn create(app: &AppHandle<Wry>, live: bool) -> tauri::Result<()> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Ok(());
    }

    let ko = is_korean();
    let status = MenuItem::with_id(
        app,
        "tray_status",
        status_text(live, ko),
        false,
        None::<&str>,
    )?;
    let show = MenuItem::with_id(
        app,
        "show",
        if ko { "창 열기" } else { "Show Window" },
        true,
        None::<&str>,
    )?;
    let start = MenuItem::with_id(
        app,
        "start_server_tray",
        if ko { "서버 시작" } else { "Start Server" },
        !live,
        None::<&str>,
    )?;
    let stop = MenuItem::with_id(
        app,
        "stop_server_tray",
        if ko { "서버 중지" } else { "Stop Server" },
        live,
        None::<&str>,
    )?;
    let control = MenuItem::with_id(
        app,
        "open_control_tray",
        if ko { "웹 컨트롤" } else { "Web Control" },
        true,
        None::<&str>,
    )?;
    let overlay = MenuItem::with_id(
        app,
        "open_overlay_tray",
        if ko { "오버레이" } else { "Overlay" },
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        if ko { "종료" } else { "Quit" },
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &status,
            &PredefinedMenuItem::separator(app)?,
            &show,
            &start,
            &stop,
            &PredefinedMenuItem::separator(app)?,
            &control,
            &overlay,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_image(app, live))
        .tooltip(tooltip_text(live, ko))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "start_server_tray" => {
                if let Some(state) = app.try_state::<LauncherState>() {
                    let settings = LauncherSettings::load();
                    let mut controller = state.server_controller.lock();
                    if let Err(err) = controller.start(state.app_state.clone(), settings.port) {
                        eprintln!("[Tray] Failed to start server: {}", err);
                    }
                }
            }
            "stop_server_tray" => {
                if let Some(state) = app.try_state::<LauncherState>() {
                    let mut controller = state.server_controller.lock();
                    if controller.is_running() {
                        let _ = controller.stop();
                    }
                }
            }
            "open_control_tray" => crate::open_service_url("/control"),
            "open_overlay_tray" => crate::open_service_url("/overlay"),
            "quit" => {
                crate::try_stop_server(app);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left click toggles the window: the icon now lives in the tray for the
            // whole session, so it has to work both ways.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    let _ = ITEMS.set(TrayItems {
        status,
        start,
        stop,
    });

    Ok(())
}

/// Polls the server flag and repaints the tray when it flips. Polling keeps every
/// path in sync - launcher UI, tray menu, or the server going down on its own -
/// without ever taking the controller lock from the main thread.
fn spawn_watcher(app: AppHandle<Wry>, running: Arc<Mutex<bool>>, live: bool) {
    // Start out of sync so the first tick also paints the taskbar overlay, which
    // cannot be set while the event loop is still starting up.
    let mut last = !live;

    std::thread::spawn(move || loop {
        let now = *running.lock();
        if now != last {
            last = now;
            apply(&app, now);
        }
        std::thread::sleep(Duration::from_millis(300));
    });
}

fn apply(app: &AppHandle<Wry>, live: bool) {
    let ko = is_korean();

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(tray_image(app, live)));
        let _ = tray.set_tooltip(Some(tooltip_text(live, ko)));
    }

    if let Some(items) = ITEMS.get() {
        let _ = items.status.set_text(status_text(live, ko));
        let _ = items.start.set_enabled(!live);
        let _ = items.stop.set_enabled(live);
    }

    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window("main") {
        let overlay = if live {
            let (rgba, size) = OVERLAY_LIVE.get_or_init(|| (build_overlay_dot(32), 32));
            Some(Image::new(rgba, *size, *size))
        } else {
            None
        };
        let _ = window.set_overlay_icon(overlay);
    }

    // The taskbar overlay has no counterpart elsewhere; the closest equivalent is a
    // badge on the macOS dock tile or the Linux launcher entry.
    #[cfg(not(target_os = "windows"))]
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_badge_count(if live { Some(1) } else { None });
    }
}

fn show_main_window(app: &AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_skip_taskbar(false);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_main_window(app: &AppHandle<Wry>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let hidden = !window.is_visible().unwrap_or(true) || window.is_minimized().unwrap_or(false);
    if hidden {
        show_main_window(app);
    } else {
        let _ = window.hide();
        let _ = window.set_skip_taskbar(true);
    }
}

fn is_korean() -> bool {
    LauncherSettings::load().language != "en"
}

fn status_text(live: bool, ko: bool) -> String {
    if !live {
        return if ko {
            "서버 정지됨"
        } else {
            "Server stopped"
        }
        .to_string();
    }

    let port = LauncherSettings::load().port;
    if ko {
        format!("서버 실행 중 · 포트 {}", port)
    } else {
        format!("Server running · port {}", port)
    }
}

fn tooltip_text(live: bool, ko: bool) -> String {
    format!("KeyQueueViewer\n{}", status_text(live, ko))
}

// ---------------------------------------------------------------------------
// Icon rendering
// ---------------------------------------------------------------------------

fn tray_image(app: &AppHandle<Wry>, live: bool) -> Image<'static> {
    let cell = if live { &ICON_LIVE } else { &ICON_IDLE };
    let (rgba, size) = cell.get_or_init(|| {
        let (mut rgba, size) = base_icon(app);
        if live {
            draw_live_dot(&mut rgba, size);
        }
        (rgba, size)
    });
    Image::new(rgba, *size, *size)
}

fn base_icon(app: &AppHandle<Wry>) -> (Vec<u8>, u32) {
    let target = tray_icon_size();

    if let Some((rgba, size)) = load_ico_entry(target) {
        if size == target {
            return (rgba, target);
        }
        return (resize(&rgba, size, target), target);
    }

    // The bundled .ico should always decode; fall back to whatever the window was
    // given rather than dropping the tray icon entirely.
    match app.default_window_icon() {
        Some(icon) if icon.width() == icon.height() => (icon.rgba().to_vec(), icon.width()),
        _ => (vec![0; (target * target * 4) as usize], target),
    }
}

#[cfg(target_os = "windows")]
fn tray_icon_size() -> u32 {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

    let size = unsafe { GetSystemMetrics(SM_CXSMICON) };
    if size <= 0 {
        16
    } else {
        (size as u32).clamp(16, 64)
    }
}

#[cfg(not(target_os = "windows"))]
fn tray_icon_size() -> u32 {
    32
}

/// Pulls the smallest square 32-bit entry of the bundled .ico that is at least
/// `target` wide. PNG-compressed entries are skipped - decoding them would mean
/// pulling in an image decoder for no gain, since every small size is a bitmap.
fn load_ico_entry(target: u32) -> Option<(Vec<u8>, u32)> {
    if ICON_ICO.len() < 6 || u16::from_le_bytes([ICON_ICO[2], ICON_ICO[3]]) != 1 {
        return None;
    }

    let count = u16::from_le_bytes([ICON_ICO[4], ICON_ICO[5]]) as usize;
    let mut best: Option<(u32, usize, usize)> = None;

    for i in 0..count {
        let entry = 6 + i * 16;
        if entry + 16 > ICON_ICO.len() {
            break;
        }

        let width = match ICON_ICO[entry] {
            0 => 256,
            w => u32::from(w),
        };
        let height = match ICON_ICO[entry + 1] {
            0 => 256,
            h => u32::from(h),
        };
        if width != height {
            continue;
        }

        let len = read_u32(ICON_ICO, entry + 8) as usize;
        let offset = read_u32(ICON_ICO, entry + 12) as usize;
        if len == 0 || offset.saturating_add(len) > ICON_ICO.len() {
            continue;
        }
        if ICON_ICO[offset..].starts_with(b"\x89PNG") {
            continue;
        }

        let better = match best {
            None => true,
            Some((current, _, _)) if current >= target => width >= target && width < current,
            Some((current, _, _)) => width > current,
        };
        if better {
            best = Some((width, offset, len));
        }
    }

    let (size, offset, len) = best?;
    decode_bmp32(offset, len, size).map(|rgba| (rgba, size))
}

/// Decodes the BITMAPINFOHEADER + BGRA payload of one .ico entry. The AND mask
/// that follows is ignored: every entry here is 32-bit with a real alpha channel.
fn decode_bmp32(offset: usize, len: usize, size: u32) -> Option<Vec<u8>> {
    let header = read_u32(ICON_ICO, offset) as usize;
    if header < 40 || header > len {
        return None;
    }
    if u16::from_le_bytes([ICON_ICO[offset + 14], ICON_ICO[offset + 15]]) != 32 {
        return None;
    }
    if read_u32(ICON_ICO, offset + 16) != 0 {
        return None; // compressed payload
    }

    let stride = size as usize * 4;
    let start = offset + header;
    if start + stride * size as usize > offset + len {
        return None;
    }

    let mut rgba = vec![0u8; stride * size as usize];
    for y in 0..size as usize {
        // ICO bitmaps are stored bottom-up.
        let src = start + (size as usize - 1 - y) * stride;
        for x in 0..size as usize {
            let s = src + x * 4;
            let d = y * stride + x * 4;
            rgba[d] = ICON_ICO[s + 2];
            rgba[d + 1] = ICON_ICO[s + 1];
            rgba[d + 2] = ICON_ICO[s];
            rgba[d + 3] = ICON_ICO[s + 3];
        }
    }
    Some(rgba)
}

/// Area-average downscale. Colors are weighted by alpha so transparent pixels do
/// not bleed dark fringes into the edges.
fn resize(src: &[u8], src_size: u32, dst_size: u32) -> Vec<u8> {
    let scale = src_size as f32 / dst_size as f32;
    let mut out = vec![0u8; (dst_size * dst_size * 4) as usize];

    for dy in 0..dst_size {
        let y0 = (dy as f32 * scale) as u32;
        let y1 = (((dy + 1) as f32 * scale).ceil() as u32).clamp(y0 + 1, src_size);
        for dx in 0..dst_size {
            let x0 = (dx as f32 * scale) as u32;
            let x1 = (((dx + 1) as f32 * scale).ceil() as u32).clamp(x0 + 1, src_size);

            let mut alpha = 0.0f32;
            let mut color = [0.0f32; 3];
            let mut count = 0.0f32;

            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * src_size + x) * 4) as usize;
                    let a = f32::from(src[i + 3]) / 255.0;
                    for (c, channel) in color.iter_mut().enumerate() {
                        *channel += f32::from(src[i + c]) * a;
                    }
                    alpha += a;
                    count += 1.0;
                }
            }

            let d = ((dy * dst_size + dx) * 4) as usize;
            if alpha > 0.0 {
                for c in 0..3 {
                    out[d + c] = (color[c] / alpha).round().clamp(0.0, 255.0) as u8;
                }
            }
            out[d + 3] = (alpha / count * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    out
}

/// Composites the live dot into the bottom-right corner. A fully transparent gap
/// around it keeps the dot readable whatever the icon or the taskbar sits behind.
fn draw_live_dot(rgba: &mut [u8], size: u32) {
    let edge = size as f32;
    let radius = (edge * 0.22).max(2.5);
    let gap = (edge * 0.075).max(1.0);
    let center = edge - radius - gap;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let cleared = (radius + gap + 0.5 - dist).clamp(0.0, 1.0);
            if cleared <= 0.0 {
                continue;
            }

            let i = ((y * size + x) * 4) as usize;
            let under = f32::from(rgba[i + 3]) / 255.0 * (1.0 - cleared);
            let cover = (radius + 0.5 - dist).clamp(0.0, 1.0);
            let alpha = cover + under * (1.0 - cover);

            if alpha <= 0.0 {
                rgba[i..i + 4].fill(0);
                continue;
            }
            for c in 0..3 {
                let dot = f32::from(DOT_RGB[c]) * cover;
                let base = f32::from(rgba[i + c]) * under * (1.0 - cover);
                rgba[i + c] = ((dot + base) / alpha).round().clamp(0.0, 255.0) as u8;
            }
            rgba[i + 3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// The standalone dot Windows paints over the taskbar button. It carries a light
/// ring so it stays separate from whatever the app icon shows underneath.
#[cfg(target_os = "windows")]
fn build_overlay_dot(size: u32) -> Vec<u8> {
    let edge = size as f32;
    let center = edge / 2.0;
    let outer = edge * 0.48;
    let inner = edge * 0.37;

    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let alpha = (outer + 0.5 - dist).clamp(0.0, 1.0);
            if alpha <= 0.0 {
                continue;
            }
            let fill = (inner + 0.5 - dist).clamp(0.0, 1.0);

            let i = ((y * size + x) * 4) as usize;
            for c in 0..3 {
                let color = f32::from(RING_RGB[c]) * (1.0 - fill) + f32::from(DOT_RGB[c]) * fill;
                rgba[i + c] = color.round().clamp(0.0, 255.0) as u8;
            }
            rgba[i + 3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    rgba
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

// ---------------------------------------------------------------------------
// Windows 11 notification area promotion
// ---------------------------------------------------------------------------

/// Windows 11 files every tray icon under HKCU\Control Panel\NotifyIconSettings
/// and buries new ones in the overflow flyout, where a status dot is worthless.
/// Flip IsPromoted once so the icon sits on the taskbar itself; the user stays in
/// control afterwards because we never touch it again.
#[cfg(target_os = "windows")]
fn spawn_promotion() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    const FLAG_PATH: &str = r"Software\KeyViewer";
    const FLAG_NAME: &str = "TrayPromoted";
    const SETTINGS_PATH: &str = r"Control Panel\NotifyIconSettings";

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(FLAG_PATH) {
        if key.get_value::<u32, _>(FLAG_NAME).unwrap_or(0) != 0 {
            return;
        }
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let exe = exe.to_string_lossy().to_lowercase();

    std::thread::spawn(move || {
        // Explorer only writes the entry once it has seen the icon, so give it a
        // few tries before giving up for this session.
        for delay in [2u64, 5, 10, 20] {
            std::thread::sleep(Duration::from_secs(delay));

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let Ok(settings) = hkcu.open_subkey(SETTINGS_PATH) else {
                return; // Windows 10 and older have no such key
            };

            let mut promoted = false;
            for name in settings.enum_keys().flatten() {
                let Ok(entry) = settings.open_subkey_with_flags(&name, KEY_READ | KEY_SET_VALUE)
                else {
                    continue;
                };
                let path: String = match entry.get_value("ExecutablePath") {
                    Ok(path) => path,
                    Err(_) => continue,
                };
                if path.to_lowercase() != exe {
                    continue;
                }
                if entry.set_value("IsPromoted", &1u32).is_ok() {
                    promoted = true;
                }
            }

            if promoted {
                if let Ok((key, _)) = hkcu.create_subkey(FLAG_PATH) {
                    let _ = key.set_value(FLAG_NAME, &1u32);
                }
                println!("[Tray] Promoted tray icon to the notification area");
                return;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    }

    #[test]
    fn picks_the_smallest_entry_that_covers_the_target() {
        assert_eq!(load_ico_entry(16).map(|(_, size)| size), Some(16));
        assert_eq!(load_ico_entry(20).map(|(_, size)| size), Some(24));
        assert_eq!(load_ico_entry(32).map(|(_, size)| size), Some(32));
        // Larger than every bitmap entry: fall back to the biggest one available.
        assert_eq!(load_ico_entry(512).map(|(_, size)| size), Some(128));
    }

    #[test]
    fn decodes_opaque_pixels() {
        let (rgba, size) = load_ico_entry(32).expect("32px entry");
        assert_eq!(rgba.len(), (size * size * 4) as usize);
        // The glyph fills the whole tile, so the center has to be opaque.
        assert_eq!(pixel(&rgba, size, size / 2, size / 2)[3], 255);
    }

    #[test]
    fn resize_keeps_the_pixel_count_and_the_alpha() {
        let (rgba, size) = load_ico_entry(24).expect("24px entry");
        let scaled = resize(&rgba, size, 20);
        assert_eq!(scaled.len(), 20 * 20 * 4);
        assert_eq!(pixel(&scaled, 20, 10, 10)[3], 255);
    }

    #[test]
    fn live_dot_lands_in_the_bottom_right_corner() {
        let (mut rgba, size) = load_ico_entry(32).expect("32px entry");
        let before = pixel(&rgba, size, 4, 4);
        draw_live_dot(&mut rgba, size);

        // Opposite corner is untouched.
        assert_eq!(pixel(&rgba, size, 4, 4), before);

        // Dot center sits at `size - radius - gap` and has to be solid red.
        let center = (size as f32 - size as f32 * 0.22 - size as f32 * 0.075) as u32;
        let dot = pixel(&rgba, size, center, center);
        assert_eq!([dot[0], dot[1], dot[2]], DOT_RGB);
        assert_eq!(dot[3], 255);

        // The gap around the dot is punched all the way through.
        assert_eq!(pixel(&rgba, size, size - 1, center)[3], 0);
    }
}
