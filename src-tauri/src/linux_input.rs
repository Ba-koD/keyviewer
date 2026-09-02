// Kernel-level key capture for Linux.
//
// rdev's Linux backend records X11 events through XRecord, which only ever sees
// what the X server delivers - on a Wayland session that is nothing but XWayland
// traffic, so native Wayland keystrokes are invisible. Reading /dev/input/event*
// puts us below the display server instead, so the same path works on X11, on
// Wayland and on a bare TTY.
//
// The price is permissions: the device nodes are root:input mode 0660, so the user
// has to be in the `input` group. When they are not, `status()` reports why and the
// launcher surfaces it instead of silently capturing nothing.

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use serde::Serialize;

use crate::state::AppState;
use crate::window_info;

const EV_KEY: u16 = 1;

/// `struct input_event` is a `timeval` (two longs) followed by type, code and
/// value. Correct for the 64-bit targets the release workflow builds.
const EVENT_SIZE: usize = std::mem::size_of::<usize>() * 2 + 8;

/// EV_KEY bit in the `B: EV=` bitmask of /proc/bus/input/devices.
const EV_KEY_BIT: u64 = 1 << 1;

#[derive(Clone, Debug, Serialize)]
pub struct InputStatus {
    /// Which capture path is live: "evdev", "x11", or "none".
    pub backend: String,
    pub ok: bool,
    /// Human readable reason, shown by the launcher when `ok` is false.
    pub detail: String,
    pub devices: usize,
}

static STATUS: OnceLock<Mutex<InputStatus>> = OnceLock::new();

fn status_cell() -> &'static Mutex<InputStatus> {
    STATUS.get_or_init(|| {
        Mutex::new(InputStatus {
            backend: "none".to_string(),
            ok: false,
            detail: String::new(),
            devices: 0,
        })
    })
}

pub fn status() -> InputStatus {
    status_cell().lock().clone()
}

pub fn set_status(backend: &str, ok: bool, detail: String, devices: usize) {
    let mut current = status_cell().lock();
    *current = InputStatus {
        backend: backend.to_string(),
        ok,
        detail,
        devices,
    };
}

/// One device block of /proc/bus/input/devices that can produce key events.
#[derive(Debug, PartialEq, Eq)]
pub struct InputDevice {
    pub name: String,
    pub event: String,
}

/// Picks the event nodes that report EV_KEY. That covers keyboards and mouse
/// buttons and leaves out pure motion devices such as touchpads and gamepads.
pub fn parse_input_devices(contents: &str) -> Vec<InputDevice> {
    let mut devices = Vec::new();

    for block in contents.split("\n\n") {
        let mut name = String::new();
        let mut event = None;
        let mut has_keys = false;

        for line in block.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("N: Name=") {
                name = rest.trim().trim_matches('"').to_string();
            } else if let Some(rest) = line.strip_prefix("H: Handlers=") {
                event = rest
                    .split_whitespace()
                    .find(|handler| {
                        handler.starts_with("event") && handler[5..].parse::<u32>().is_ok()
                    })
                    .map(str::to_string);
            } else if let Some(rest) = line.strip_prefix("B: EV=") {
                has_keys = u64::from_str_radix(rest.trim(), 16)
                    .map(|bits| bits & EV_KEY_BIT != 0)
                    .unwrap_or(false);
            }
        }

        if let (true, Some(event)) = (has_keys, event) {
            devices.push(InputDevice { name, event });
        }
    }

    devices
}

/// Splits one `input_event` record into (type, code, value).
pub fn decode_event(buf: &[u8]) -> Option<(u16, u16, i32)> {
    if buf.len() < EVENT_SIZE {
        return None;
    }
    let tail = &buf[EVENT_SIZE - 8..EVENT_SIZE];
    Some((
        u16::from_ne_bytes([tail[0], tail[1]]),
        u16::from_ne_bytes([tail[2], tail[3]]),
        i32::from_ne_bytes([tail[4], tail[5], tail[6], tail[7]]),
    ))
}

/// Linux input event codes, mapped to the same labels the X11 path produces.
pub fn label_for(code: u16) -> String {
    match code {
        1 => "ESC",
        2 => "1",
        3 => "2",
        4 => "3",
        5 => "4",
        6 => "5",
        7 => "6",
        8 => "7",
        9 => "8",
        10 => "9",
        11 => "0",
        12 => "-",
        13 => "=",
        14 => "BKSP",
        15 => "TAB",
        16 => "Q",
        17 => "W",
        18 => "E",
        19 => "R",
        20 => "T",
        21 => "Y",
        22 => "U",
        23 => "I",
        24 => "O",
        25 => "P",
        26 => "[",
        27 => "]",
        28 => "ENTER",
        29 => "LCTRL",
        30 => "A",
        31 => "S",
        32 => "D",
        33 => "F",
        34 => "G",
        35 => "H",
        36 => "J",
        37 => "K",
        38 => "L",
        39 => ";",
        40 => "'",
        41 => "`",
        42 => "LSHIFT",
        43 => "\\",
        44 => "Z",
        45 => "X",
        46 => "C",
        47 => "V",
        48 => "B",
        49 => "N",
        50 => "M",
        51 => ",",
        52 => ".",
        53 => "/",
        54 => "RSHIFT",
        55 => "*",
        56 => "LALT",
        57 => "SPACE",
        58 => "CAPS",
        59 => "F1",
        60 => "F2",
        61 => "F3",
        62 => "F4",
        63 => "F5",
        64 => "F6",
        65 => "F7",
        66 => "F8",
        67 => "F9",
        68 => "F10",
        69 => "NUM",
        70 => "SCROLL",
        71 => "7",
        72 => "8",
        73 => "9",
        74 => "-",
        75 => "4",
        76 => "5",
        77 => "6",
        78 => "+",
        79 => "1",
        80 => "2",
        81 => "3",
        82 => "0",
        83 => "DEL",
        87 => "F11",
        88 => "F12",
        96 => "ENTER",
        97 => "RCTRL",
        98 => "/",
        99 => "PRINT",
        100 => "RALT",
        102 => "HOME",
        103 => "UP",
        104 => "PG UP",
        105 => "LEFT",
        106 => "RIGHT",
        107 => "END",
        108 => "DOWN",
        109 => "PG DN",
        110 => "INS",
        111 => "DEL",
        119 => "PAUSE",
        125 => "LSUPER",
        126 => "RSUPER",
        127 => "MENU",
        272 => "LMB",
        273 => "RMB",
        274 => "MMB",
        275 => "MB4",
        276 => "MB5",
        other => return format!("KEY{}", other),
    }
    .to_string()
}

/// Opens every key-capable device, reporting which ones were refused so the
/// launcher can tell the user exactly what to fix.
fn open_devices(devices: &[InputDevice]) -> (Vec<(String, File)>, usize) {
    let mut opened = Vec::new();
    let mut denied = 0;

    for device in devices {
        let path = format!("/dev/input/{}", device.event);
        match File::open(&path) {
            Ok(file) => opened.push((device.name.clone(), file)),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => denied += 1,
            Err(e) => eprintln!("[Keyboard Hook] Skipping {}: {}", path, e),
        }
    }

    (opened, denied)
}

fn read_device(name: String, mut file: File, state: Arc<RwLock<AppState>>) {
    let mut buf = vec![0u8; EVENT_SIZE];

    loop {
        if let Err(e) = file.read_exact(&mut buf) {
            eprintln!("[Keyboard Hook] {} stopped: {}", name, e);
            return;
        }

        let Some((kind, code, value)) = decode_event(&buf) else {
            continue;
        };
        if kind != EV_KEY {
            continue;
        }

        match value {
            // 2 is auto-repeat; the key is already down so there is nothing to add.
            1 => {
                let label = label_for(code);
                let source_window = window_info::get_foreground_window();
                state
                    .write()
                    .add_key_with_window(u32::from(code), label, source_window);
            }
            0 => state.write().remove_key(u32::from(code)),
            _ => {}
        }
    }
}

/// Starts one reader thread per device and rescans periodically so a keyboard
/// plugged in later starts working without a restart.
pub fn listen(state: Arc<RwLock<AppState>>) -> bool {
    let contents = match std::fs::read_to_string("/proc/bus/input/devices") {
        Ok(contents) => contents,
        Err(e) => {
            set_status(
                "none",
                false,
                format!("Could not read /proc/bus/input/devices: {}", e),
                0,
            );
            return false;
        }
    };

    let devices = parse_input_devices(&contents);
    let (opened, denied) = open_devices(&devices);

    if opened.is_empty() {
        let detail = if denied > 0 {
            format!(
                "No permission to read {} input device(s). Add your user to the `input` group \
                 (sudo usermod -aG input $USER) and log back in.",
                denied
            )
        } else {
            "No key-capable input devices were found under /dev/input.".to_string()
        };
        set_status("none", false, detail, 0);
        return false;
    }

    let detail = if denied > 0 {
        format!(
            "Reading {} device(s); {} refused. Add your user to the `input` group to capture \
             every keyboard.",
            opened.len(),
            denied
        )
    } else {
        String::new()
    };
    set_status("evdev", denied == 0, detail, opened.len());

    let mut watched: HashSet<String> = devices
        .iter()
        .map(|device| device.event.clone())
        .collect::<HashSet<_>>();

    for (name, file) in opened {
        let state = state.clone();
        std::thread::spawn(move || read_device(name, file, state));
    }

    // Rescan for hot-plugged devices.
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));

        let Ok(contents) = std::fs::read_to_string("/proc/bus/input/devices") else {
            continue;
        };
        let fresh: Vec<InputDevice> = parse_input_devices(&contents)
            .into_iter()
            .filter(|device| !watched.contains(&device.event))
            .collect();
        if fresh.is_empty() {
            continue;
        }

        let (opened, _) = open_devices(&fresh);
        for device in &fresh {
            watched.insert(device.event.clone());
        }
        for (name, file) in opened {
            eprintln!("[Keyboard Hook] Picked up new input device: {}", name);
            let state = state.clone();
            std::thread::spawn(move || read_device(name, file, state));
        }
    });

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "I: Bus=0011 Vendor=0001 Product=0001 Version=ab41\n\
N: Name=\"AT Translated Set 2 keyboard\"\n\
P: Phys=isa0060/serio0/input0\n\
H: Handlers=sysrq kbd event3 leds\n\
B: EV=120013\n\
\n\
I: Bus=0019 Vendor=0000 Product=0006 Version=0000\n\
N: Name=\"Video Bus\"\n\
H: Handlers=kbd event5\n\
B: EV=3\n\
\n\
I: Bus=0003 Vendor=046d Product=c52b Version=0111\n\
N: Name=\"Logitech USB Receiver Mouse\"\n\
H: Handlers=mouse0 event6\n\
B: EV=17\n\
\n\
I: Bus=0003 Vendor=8087 Product=0a2b Version=0001\n\
N: Name=\"Accelerometer\"\n\
H: Handlers=event9\n\
B: EV=9\n";

    #[test]
    fn keeps_only_devices_that_report_key_events() {
        let devices = parse_input_devices(SAMPLE);
        let events: Vec<&str> = devices.iter().map(|d| d.event.as_str()).collect();

        // EV=9 has no EV_KEY bit, so the accelerometer is dropped.
        assert_eq!(events, ["event3", "event5", "event6"]);
        assert_eq!(devices[0].name, "AT Translated Set 2 keyboard");
    }

    #[test]
    fn ignores_blocks_without_an_event_handler() {
        let devices = parse_input_devices("N: Name=\"No node\"\nH: Handlers=kbd\nB: EV=120013\n");
        assert!(devices.is_empty());
    }

    #[test]
    fn decodes_a_key_press_record() {
        let mut buf = vec![0u8; EVENT_SIZE];
        let tail = EVENT_SIZE - 8;
        buf[tail..tail + 2].copy_from_slice(&EV_KEY.to_ne_bytes());
        buf[tail + 2..tail + 4].copy_from_slice(&30u16.to_ne_bytes());
        buf[tail + 4..tail + 8].copy_from_slice(&1i32.to_ne_bytes());

        assert_eq!(decode_event(&buf), Some((EV_KEY, 30, 1)));
        assert_eq!(decode_event(&buf[..EVENT_SIZE - 1]), None);
    }

    #[test]
    fn labels_match_the_x11_naming() {
        assert_eq!(label_for(30), "A");
        assert_eq!(label_for(57), "SPACE");
        assert_eq!(label_for(28), "ENTER");
        assert_eq!(label_for(96), "ENTER"); // numpad enter
        assert_eq!(label_for(272), "LMB");
        assert_eq!(label_for(60000), "KEY60000");
    }
}
