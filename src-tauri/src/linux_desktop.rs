// Active window lookup on Linux.
//
// X11 answers this with XGetInputFocus, but Wayland deliberately refuses to tell a
// client what any other window is doing. There is no portable replacement: the
// wlr-foreign-toplevel/ext-foreign-toplevel protocols only exist on wlroots-based
// compositors, and KWin and Mutter expose neither. So each desktop gets the route
// its own maintainers ship, and anything unrecognised falls back to XWayland.
//
//   Hyprland  hyprctl activewindow -j          (built in)
//   Sway      swaymsg -t get_tree              (built in)
//   KDE       kdotool getactivewindow          (needs kdotool installed)
//   GNOME     gdbus -> WindowsExt              (needs the window-calls-extended
//                                               shell extension)

use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::window_info::WindowInfo;

pub fn is_wayland() -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return true;
    }
    std::env::var("XDG_SESSION_TYPE")
        .map(|kind| kind.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
}

/// Reads a process name out of /proc so the "process" target mode has something to
/// match on. The X11 path leaves this empty today.
pub fn process_name(pid: u32) -> String {
    std::fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|name| name.trim().to_string())
        .unwrap_or_default()
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Walks a JSON value for the node the compositor marked as focused.
pub fn parse_sway_tree(json: &str) -> Option<WindowInfo> {
    fn focused(node: &serde_json::Value) -> Option<&serde_json::Value> {
        if node.get("focused").and_then(serde_json::Value::as_bool) == Some(true) {
            return Some(node);
        }
        for key in ["nodes", "floating_nodes"] {
            for child in node.get(key)?.as_array()? {
                if let Some(found) = focused(child) {
                    return Some(found);
                }
            }
        }
        None
    }

    let tree: serde_json::Value = serde_json::from_str(json).ok()?;
    let node = focused(&tree)?;

    let pid = node.get("pid").and_then(serde_json::Value::as_u64);
    Some(WindowInfo {
        hwnd: node
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        title: node
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        process: pid.map(|pid| process_name(pid as u32)).unwrap_or_default(),
        class: node
            .get("app_id")
            .and_then(serde_json::Value::as_str)
            .or_else(|| node.get("window_properties")?.get("class")?.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}

pub fn parse_hyprctl_window(json: &str) -> Option<WindowInfo> {
    let window: serde_json::Value = serde_json::from_str(json).ok()?;
    let class = window.get("class")?.as_str()?.to_string();
    let pid = window.get("pid").and_then(serde_json::Value::as_u64);

    Some(WindowInfo {
        hwnd: window
            .get("address")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        title: window
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        process: pid.map(|pid| process_name(pid as u32)).unwrap_or_default(),
        class,
    })
}

/// gdbus prints replies as a tuple literal, e.g. `('Firefox',)` or `(1234,)`.
pub fn parse_gdbus_scalar(reply: &str) -> Option<String> {
    let inner = reply.trim().strip_prefix('(')?.strip_suffix(')')?;
    let inner = inner.trim().trim_end_matches(',').trim();
    let unquoted = inner
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
        .unwrap_or(inner);
    if unquoted.is_empty() {
        None
    } else {
        Some(unquoted.to_string())
    }
}

fn hyprland() -> Option<WindowInfo> {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
    parse_hyprctl_window(&run("hyprctl", &["activewindow", "-j"])?)
}

fn sway() -> Option<WindowInfo> {
    std::env::var_os("SWAYSOCK")?;
    parse_sway_tree(&run("swaymsg", &["-t", "get_tree"])?)
}

fn gnome() -> Option<WindowInfo> {
    let call = |method: &str| {
        run(
            "gdbus",
            &[
                "call",
                "--session",
                "--dest",
                "org.gnome.Shell",
                "--object-path",
                "/org/gnome/Shell/Extensions/WindowsExt",
                "--method",
                method,
            ],
        )
        .as_deref()
        .and_then(parse_gdbus_scalar)
    };

    let class = call("org.gnome.Shell.Extensions.WindowsExt.FocusClass")?;
    let title = call("org.gnome.Shell.Extensions.WindowsExt.FocusTitle").unwrap_or_default();
    let process = call("org.gnome.Shell.Extensions.WindowsExt.FocusPID")
        .and_then(|pid| pid.parse::<u32>().ok())
        .map(process_name)
        .unwrap_or_default();

    Some(WindowInfo {
        hwnd: String::new(),
        title,
        process,
        class,
    })
}

fn kde() -> Option<WindowInfo> {
    let id = run("kdotool", &["getactivewindow"])?;
    Some(WindowInfo {
        title: run("kdotool", &["getwindowname", &id]).unwrap_or_default(),
        class: run("kdotool", &["getwindowclassname", &id]).unwrap_or_default(),
        process: run("kdotool", &["getwindowpid", &id])
            .and_then(|pid| pid.parse::<u32>().ok())
            .map(process_name)
            .unwrap_or_default(),
        hwnd: id,
    })
}

/// Tries each compositor route in turn. Returns None when none of them answered,
/// which leaves the caller on the XWayland fallback.
pub fn foreground_window() -> Option<WindowInfo> {
    hyprland().or_else(sway).or_else(gnome).or_else(kde)
}

/// Every route above forks a helper process, and the keyboard hook asks for the
/// foreground window on each key press. Hold the answer briefly so a burst of
/// typing costs one lookup instead of one per keystroke.
pub fn foreground_window_cached() -> Option<WindowInfo> {
    /// When the last lookup ran, and what it found.
    type Cached = Option<(Instant, Option<WindowInfo>)>;

    const TTL: Duration = Duration::from_millis(150);
    static CACHE: OnceLock<Mutex<Cached>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| Mutex::new(None));
    {
        let cached = cache.lock();
        if let Some((at, window)) = cached.as_ref() {
            if at.elapsed() < TTL {
                return window.clone();
            }
        }
    }

    let window = foreground_window();
    *cache.lock() = Some((Instant::now(), window.clone()));
    window
}

// ---------------------------------------------------------------------------
// Window list
// ---------------------------------------------------------------------------

pub fn parse_hyprctl_clients(json: &str) -> Vec<WindowInfo> {
    let Ok(serde_json::Value::Array(clients)) = serde_json::from_str::<serde_json::Value>(json)
    else {
        return Vec::new();
    };

    clients
        .iter()
        .filter_map(|client| {
            let class = client.get("class")?.as_str()?;
            if class.is_empty() {
                return None;
            }
            Some(WindowInfo {
                hwnd: client
                    .get("address")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                title: client
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                process: client
                    .get("pid")
                    .and_then(serde_json::Value::as_u64)
                    .map(|pid| process_name(pid as u32))
                    .unwrap_or_default(),
                class: class.to_string(),
            })
        })
        .collect()
}

/// Collects every leaf node the sway tree calls a window.
pub fn parse_sway_tree_all(json: &str) -> Vec<WindowInfo> {
    fn walk(node: &serde_json::Value, out: &mut Vec<WindowInfo>) {
        let is_window = node.get("pid").is_some()
            && node
                .get("name")
                .map(|name| !name.is_null())
                .unwrap_or(false);

        if is_window {
            let class = node
                .get("app_id")
                .and_then(serde_json::Value::as_str)
                .or_else(|| node.get("window_properties")?.get("class")?.as_str())
                .unwrap_or_default();
            out.push(WindowInfo {
                hwnd: node
                    .get("id")
                    .and_then(serde_json::Value::as_i64)
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                title: node
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                process: node
                    .get("pid")
                    .and_then(serde_json::Value::as_u64)
                    .map(|pid| process_name(pid as u32))
                    .unwrap_or_default(),
                class: class.to_string(),
            });
        }

        for key in ["nodes", "floating_nodes"] {
            if let Some(children) = node.get(key).and_then(serde_json::Value::as_array) {
                for child in children {
                    walk(child, out);
                }
            }
        }
    }

    let Ok(tree) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

/// The WindowsExt extension answers List with a JSON string wrapped in a gdbus
/// tuple, so the payload has to be unwrapped before it parses.
pub fn parse_gnome_window_list(reply: &str) -> Vec<WindowInfo> {
    let Some(payload) = parse_gdbus_scalar(reply) else {
        return Vec::new();
    };
    let Ok(serde_json::Value::Array(windows)) = serde_json::from_str::<serde_json::Value>(&payload)
    else {
        return Vec::new();
    };

    windows
        .iter()
        .filter_map(|window| {
            let class = window
                .get("wm_class")
                .and_then(serde_json::Value::as_str)
                .filter(|class| !class.is_empty())?;
            Some(WindowInfo {
                hwnd: window
                    .get("id")
                    .and_then(serde_json::Value::as_i64)
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                title: window
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                process: window
                    .get("pid")
                    .and_then(serde_json::Value::as_u64)
                    .map(|pid| process_name(pid as u32))
                    .unwrap_or_default(),
                class: class.to_string(),
            })
        })
        .collect()
}

/// The compositor-specific window list, matching `foreground_window`.
pub fn all_windows() -> Vec<WindowInfo> {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        if let Some(json) = run("hyprctl", &["clients", "-j"]) {
            let windows = parse_hyprctl_clients(&json);
            if !windows.is_empty() {
                return windows;
            }
        }
    }

    if std::env::var_os("SWAYSOCK").is_some() {
        if let Some(json) = run("swaymsg", &["-t", "get_tree"]) {
            let windows = parse_sway_tree_all(&json);
            if !windows.is_empty() {
                return windows;
            }
        }
    }

    if let Some(reply) = run(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell/Extensions/WindowsExt",
            "--method",
            "org.gnome.Shell.Extensions.WindowsExt.List",
        ],
    ) {
        let windows = parse_gnome_window_list(&reply);
        if !windows.is_empty() {
            return windows;
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_focused_node_in_a_sway_tree() {
        let json = r#"{
            "id": 1, "focused": false, "nodes": [
                { "id": 2, "focused": false, "nodes": [
                    { "id": 7, "focused": true, "name": "nvim", "app_id": "foot", "pid": 4242 }
                ], "floating_nodes": [] }
            ], "floating_nodes": []
        }"#;

        let info = parse_sway_tree(json).unwrap();
        assert_eq!(info.hwnd, "7");
        assert_eq!(info.title, "nvim");
        assert_eq!(info.class, "foot");
    }

    #[test]
    fn falls_back_to_the_xwayland_class_in_a_sway_tree() {
        let json = r#"{
            "id": 1, "focused": false, "nodes": [
                { "id": 3, "focused": true, "name": "Steam",
                  "window_properties": { "class": "Steam" } }
            ], "floating_nodes": []
        }"#;

        assert_eq!(parse_sway_tree(json).unwrap().class, "Steam");
    }

    #[test]
    fn returns_nothing_when_no_node_is_focused() {
        assert!(
            parse_sway_tree(r#"{"id":1,"focused":false,"nodes":[],"floating_nodes":[]}"#).is_none()
        );
    }

    #[test]
    fn reads_a_hyprctl_window() {
        let json = r#"{"address":"0x55a1","class":"firefox","title":"Docs","pid":991}"#;
        let info = parse_hyprctl_window(json).unwrap();

        assert_eq!(info.hwnd, "0x55a1");
        assert_eq!(info.class, "firefox");
        assert_eq!(info.title, "Docs");
    }

    #[test]
    fn hyprctl_without_a_class_is_not_a_window() {
        // hyprctl prints `{}` when nothing is focused.
        assert!(parse_hyprctl_window("{}").is_none());
    }

    #[test]
    fn lists_hyprland_clients_and_drops_unmapped_ones() {
        let json = r#"[
            {"address":"0x1","class":"firefox","title":"Docs","pid":1},
            {"address":"0x2","class":"","title":"","pid":2}
        ]"#;

        let windows = parse_hyprctl_clients(json);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].class, "firefox");
    }

    #[test]
    fn lists_every_window_in_a_sway_tree() {
        let json = r#"{
            "id": 1, "name": "root", "nodes": [
                { "id": 2, "name": "ws", "nodes": [
                    { "id": 7, "name": "nvim", "app_id": "foot", "pid": 42 }
                ], "floating_nodes": [
                    { "id": 8, "name": "mpv", "app_id": "mpv", "pid": 43 }
                ] }
            ], "floating_nodes": []
        }"#;

        let titles: Vec<String> = parse_sway_tree_all(json)
            .into_iter()
            .map(|window| window.title)
            .collect();
        assert_eq!(titles, ["nvim", "mpv"]);
    }

    #[test]
    fn reads_the_gnome_extension_window_list() {
        let reply = r#"('[{"id":1,"wm_class":"firefox","title":"Docs","pid":9}]',)"#;
        let windows = parse_gnome_window_list(reply);

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].class, "firefox");
        assert_eq!(windows[0].title, "Docs");
    }

    #[test]
    fn unwraps_gdbus_tuple_replies() {
        assert_eq!(
            parse_gdbus_scalar("('Firefox',)"),
            Some("Firefox".to_string())
        );
        assert_eq!(parse_gdbus_scalar("(1234,)"), Some("1234".to_string()));
        assert_eq!(parse_gdbus_scalar("('',)"), None);
        assert_eq!(parse_gdbus_scalar("nonsense"), None);
    }
}
