use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: String,
    pub title: String,
    pub process: String,
    pub class: String,
}

#[cfg(target_os = "windows")]
pub fn get_foreground_window() -> Option<WindowInfo> {
    use windows::Win32::System::ProcessStatus::{GetModuleBaseNameW, K32GetProcessImageFileNameW};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // Get window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        // Get process name (robust: try limited info path first)
        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let mut process_name = String::new();
        if process_id != 0 {
            // 1) Try PROCESS_QUERY_LIMITED_INFORMATION + K32GetProcessImageFileNameW
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                let mut path_buf = [0u16; 1024];
                let len = K32GetProcessImageFileNameW(handle, &mut path_buf) as usize;
                if len > 0 {
                    let full = String::from_utf16_lossy(&path_buf[..len]);
                    if let Some(name) = full.rsplit(['\\', '/']).next() {
                        process_name = name.to_string();
                    }
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Debug: K32GetProcessImageFileNameW returned empty for pid={}",
                        process_id
                    );
                }
            } else {
                #[cfg(debug_assertions)]
                eprintln!(
                    "Debug: OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) failed for pid={}",
                    process_id
                );
            }

            // 2) Fallback to older method if still empty
            if process_name.is_empty() {
                if let Ok(handle) = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    process_id,
                ) {
                    let mut name_buf = [0u16; 512];
                    let len = GetModuleBaseNameW(handle, None, &mut name_buf);
                    if len > 0 {
                        process_name = String::from_utf16_lossy(&name_buf[..len as usize]);
                    } else {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "Debug: GetModuleBaseNameW returned empty for pid={}",
                            process_id
                        );
                    }
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("Debug: OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ) failed for pid={}", process_id);
                }
            }
        }

        // Get window class name
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

        Some(WindowInfo {
            hwnd: format!("{:?}", hwnd.0),
            title,
            process: process_name,
            class: class_name,
        })
    }
}

#[cfg(target_os = "windows")]
pub fn get_all_windows() -> Vec<WindowInfo> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::ProcessStatus::{GetModuleBaseNameW, K32GetProcessImageFileNameW};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

    let mut windows = Vec::new();

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        unsafe {
            if IsWindowVisible(hwnd).as_bool() {
                // Get window title
                let mut title_buf = [0u16; 512];
                let title_len = GetWindowTextW(hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

                if !title.is_empty() {
                    // Get process name (robust: limited info first)
                    let mut process_id = 0u32;
                    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

                    let mut process_name = String::new();
                    if process_id != 0 {
                        if let Ok(handle) =
                            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
                        {
                            let mut path_buf = [0u16; 1024];
                            let len = K32GetProcessImageFileNameW(handle, &mut path_buf) as usize;
                            if len > 0 {
                                let full = String::from_utf16_lossy(&path_buf[..len]);
                                if let Some(name) = full.rsplit(['\\', '/']).next() {
                                    process_name = name.to_string();
                                }
                            } else {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "Debug: K32GetProcessImageFileNameW returned empty for pid={}",
                                    process_id
                                );
                            }
                        } else {
                            #[cfg(debug_assertions)]
                            eprintln!("Debug: OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) failed for pid={}", process_id);
                        }

                        if process_name.is_empty() {
                            if let Ok(handle) = OpenProcess(
                                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                                false,
                                process_id,
                            ) {
                                let mut name_buf = [0u16; 512];
                                let len = GetModuleBaseNameW(handle, None, &mut name_buf);
                                if len > 0 {
                                    process_name =
                                        String::from_utf16_lossy(&name_buf[..len as usize]);
                                } else {
                                    #[cfg(debug_assertions)]
                                    eprintln!(
                                        "Debug: GetModuleBaseNameW returned empty for pid={}",
                                        process_id
                                    );
                                }
                            } else {
                                #[cfg(debug_assertions)]
                                eprintln!("Debug: OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ) failed for pid={}", process_id);
                            }
                        }
                    }

                    // Get window class name
                    let mut class_buf = [0u16; 256];
                    let class_len = GetClassNameW(hwnd, &mut class_buf);
                    let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

                    windows.push(WindowInfo {
                        hwnd: format!("{:?}", hwnd.0),
                        title,
                        process: process_name,
                        class: class_name,
                    });
                }
            }
        }

        BOOL::from(true)
    }

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut windows as *mut _ as isize));
    }

    windows
}

#[cfg(target_os = "macos")]
pub fn get_foreground_window() -> Option<WindowInfo> {
    use core_foundation::array::{CFArray, CFArrayRef};
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::string::{CFString, CFStringRef};
    use std::os::raw::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
    }

    #[allow(dead_code, non_upper_case_globals)]
    const kCGWindowListOptionAll: u32 = 0;
    #[allow(non_upper_case_globals)]
    const kCGWindowListOptionOnScreenOnly: u32 = 1 << 0;

    unsafe {
        // Use kCGWindowListOptionOnScreenOnly to get visible windows
        // Don't use ExcludeDesktopElements as it filters too much
        let window_list_info = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, 0);

        if window_list_info.is_null() {
            #[cfg(debug_assertions)]
            eprintln!("Debug: CGWindowListCopyWindowInfo returned null");
            return None;
        }

        let window_list: CFArray<*const c_void> = CFArray::wrap_under_create_rule(window_list_info);

        // Get the first user-visible window (frontmost app window)
        let owner_name_key = CFString::from_static_string("kCGWindowOwnerName");
        let window_name_key = CFString::from_static_string("kCGWindowName");
        let window_number_key = CFString::from_static_string("kCGWindowNumber");
        let window_layer_key = CFString::from_static_string("kCGWindowLayer");

        for i in 0..window_list.len() {
            if let Some(item_ref) = window_list.get(i) {
                let dict_ptr: *const c_void = *item_ref;
                let window_info: CFDictionary<CFString, CFType> =
                    CFDictionary::wrap_under_get_rule(dict_ptr as CFDictionaryRef);

                // Check window layer (0 = normal application window)
                let layer = window_info
                    .find(window_layer_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let num: CFNumber =
                            CFNumber::wrap_under_get_rule(v.as_CFTypeRef() as CFNumberRef);
                        num.to_i64()
                    })
                    .unwrap_or(999);

                // Skip non-normal windows (layer != 0 means system UI, menubar, dock, etc)
                if layer != 0 {
                    continue;
                }

                let owner_name = window_info
                    .find(owner_name_key.as_concrete_TypeRef())
                    .map(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        s.to_string()
                    })
                    .unwrap_or_default();

                let window_name = window_info
                    .find(window_name_key.as_concrete_TypeRef())
                    .map(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        s.to_string()
                    })
                    .unwrap_or_default();

                let window_number = window_info
                    .find(window_number_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let num: CFNumber =
                            CFNumber::wrap_under_get_rule(v.as_CFTypeRef() as CFNumberRef);
                        num.to_i64()
                    })
                    .unwrap_or(0);

                // Debug logging to see what we're getting
                #[cfg(debug_assertions)]
                eprintln!(
                    "[macOS] Window - Layer: {}, Owner: '{}', Title: '{}', Number: {}",
                    layer, owner_name, window_name, window_number
                );

                // Static warning for empty titles (only show once per run)
                static TITLE_WARNING_SHOWN: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if window_name.is_empty()
                    && !owner_name.is_empty()
                    && !TITLE_WARNING_SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    eprintln!("\n⚠️  [macOS] Window titles are EMPTY!");
                    eprintln!("⚠️  Screen Recording permission is probably missing.");
                    eprintln!("⚠️  Fix: System Settings > Privacy & Security > Screen Recording");
                    eprintln!("⚠️  Enable 'KeyQueueViewer', then QUIT (Cmd+Q) and restart.\n");
                }

                // Return first normal window with a name (even if title is empty)
                if !owner_name.is_empty() {
                    return Some(WindowInfo {
                        hwnd: window_number.to_string(),
                        title: window_name,
                        process: owner_name,
                        class: String::new(), // macOS doesn't have window class like Windows
                    });
                }
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn get_all_windows() -> Vec<WindowInfo> {
    use core_foundation::array::{CFArray, CFArrayRef};
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::string::{CFString, CFStringRef};
    use std::os::raw::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
    }

    #[allow(non_upper_case_globals)]
    const kCGWindowListOptionOnScreenOnly: u32 = 1 << 0;

    let mut windows = Vec::new();

    unsafe {
        // Use kCGWindowListOptionOnScreenOnly to get visible windows
        // Don't use ExcludeDesktopElements as it filters too much
        let window_list_info = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, 0);

        if window_list_info.is_null() {
            #[cfg(debug_assertions)]
            eprintln!("Debug: CGWindowListCopyWindowInfo returned null");
            return windows;
        }

        let window_list: CFArray<*const c_void> = CFArray::wrap_under_create_rule(window_list_info);

        let owner_name_key = CFString::from_static_string("kCGWindowOwnerName");
        let window_name_key = CFString::from_static_string("kCGWindowName");
        let window_number_key = CFString::from_static_string("kCGWindowNumber");
        let window_layer_key = CFString::from_static_string("kCGWindowLayer");

        for i in 0..window_list.len() {
            if let Some(item_ref) = window_list.get(i) {
                let dict_ptr: *const c_void = *item_ref;
                let window_info: CFDictionary<CFString, CFType> =
                    CFDictionary::wrap_under_get_rule(dict_ptr as CFDictionaryRef);

                // Check window layer (0 = normal application window)
                let layer = window_info
                    .find(window_layer_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let num: CFNumber =
                            CFNumber::wrap_under_get_rule(v.as_CFTypeRef() as CFNumberRef);
                        num.to_i64()
                    })
                    .unwrap_or(999);

                // Skip non-normal windows (layer != 0 means system UI, menubar, dock, etc)
                if layer != 0 {
                    continue;
                }

                let owner_name = window_info
                    .find(owner_name_key.as_concrete_TypeRef())
                    .map(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        s.to_string()
                    })
                    .unwrap_or_default();

                let window_name = window_info
                    .find(window_name_key.as_concrete_TypeRef())
                    .map(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        s.to_string()
                    })
                    .unwrap_or_default();

                let window_number = window_info
                    .find(window_number_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let num: CFNumber =
                            CFNumber::wrap_under_get_rule(v.as_CFTypeRef() as CFNumberRef);
                        num.to_i64()
                    })
                    .unwrap_or(0);

                // Debug logging to see what we're getting
                #[cfg(debug_assertions)]
                eprintln!(
                    "[macOS] Window - Layer: {}, Owner: '{}', Title: '{}', Number: {}",
                    layer, owner_name, window_name, window_number
                );

                // Add windows with owner name (even if title is empty)
                // This catches browser tabs and other windows without titles
                if !owner_name.is_empty() {
                    windows.push(WindowInfo {
                        hwnd: window_number.to_string(),
                        title: if window_name.is_empty() {
                            format!("{} (No Title)", owner_name)
                        } else {
                            window_name
                        },
                        process: owner_name,
                        class: String::new(), // macOS doesn't have window class
                    });
                }
            }
        }
    }

    windows
}

// Linux: the X11 route below also covers XWayland, so it stays the fallback for
// every session. On Wayland the compositor is asked first - see linux_desktop.
#[cfg(target_os = "linux")]
pub fn get_foreground_window() -> Option<WindowInfo> {
    if crate::linux_desktop::is_wayland() {
        if let Some(info) = crate::linux_desktop::foreground_window_cached() {
            return Some(info);
        }
    }
    x11_foreground_window()
}

/// XGetInputFocus often lands on a child window that carries neither the title nor
/// the class, so walk up to the top level the window manager actually tracks.
#[cfg(target_os = "linux")]
unsafe fn x11_top_level(
    display: *mut x11::xlib::Display,
    mut window: x11::xlib::Window,
) -> x11::xlib::Window {
    use x11::xlib::*;

    for _ in 0..32 {
        let mut root: Window = 0;
        let mut parent: Window = 0;
        let mut children: *mut Window = std::ptr::null_mut();
        let mut count: u32 = 0;

        if XQueryTree(
            display,
            window,
            &mut root,
            &mut parent,
            &mut children,
            &mut count,
        ) == 0
        {
            break;
        }
        if !children.is_null() {
            XFree(children as *mut _);
        }
        if parent == 0 || parent == root {
            break;
        }
        window = parent;
    }

    window
}

#[cfg(target_os = "linux")]
unsafe fn x11_window_pid(
    display: *mut x11::xlib::Display,
    window: x11::xlib::Window,
) -> Option<u32> {
    use x11::xlib::*;

    let name = std::ffi::CString::new("_NET_WM_PID").ok()?;
    let atom = XInternAtom(display, name.as_ptr(), 1);
    if atom == 0 {
        return None;
    }

    let mut actual_type: Atom = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut data: *mut u8 = std::ptr::null_mut();

    let status = XGetWindowProperty(
        display,
        window,
        atom,
        0,
        1,
        0,
        0,
        &mut actual_type,
        &mut actual_format,
        &mut nitems,
        &mut bytes_after,
        &mut data,
    );

    if status != 0 || data.is_null() {
        return None;
    }
    let pid = if nitems > 0 && actual_format == 32 {
        Some(std::ptr::read_unaligned(data as *const u32))
    } else {
        None
    };
    XFree(data as *mut _);
    pid
}

#[cfg(target_os = "linux")]
unsafe fn x11_window_info(
    display: *mut x11::xlib::Display,
    window: x11::xlib::Window,
) -> WindowInfo {
    use x11::xlib::*;

    let mut name: *mut i8 = std::ptr::null_mut();
    XFetchName(display, window, &mut name);
    let title = if name.is_null() {
        String::new()
    } else {
        let title = std::ffi::CStr::from_ptr(name)
            .to_string_lossy()
            .into_owned();
        XFree(name as *mut _);
        title
    };

    let mut class_hint = XClassHint {
        res_name: std::ptr::null_mut(),
        res_class: std::ptr::null_mut(),
    };
    XGetClassHint(display, window, &mut class_hint);

    let class = if class_hint.res_class.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(class_hint.res_class)
            .to_string_lossy()
            .into_owned()
    };
    if !class_hint.res_name.is_null() {
        XFree(class_hint.res_name as *mut _);
    }
    if !class_hint.res_class.is_null() {
        XFree(class_hint.res_class as *mut _);
    }

    let process = x11_window_pid(display, window)
        .map(crate::linux_desktop::process_name)
        .unwrap_or_default();

    WindowInfo {
        hwnd: format!("{}", window),
        title,
        process,
        class,
    }
}

#[cfg(target_os = "linux")]
fn x11_foreground_window() -> Option<WindowInfo> {
    use x11::xlib::*;

    unsafe {
        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return None;
        }

        let mut focus_window: Window = 0;
        let mut revert_to: i32 = 0;
        XGetInputFocus(display, &mut focus_window, &mut revert_to);

        if focus_window == 0 {
            XCloseDisplay(display);
            return None;
        }

        let window = x11_top_level(display, focus_window);
        let info = x11_window_info(display, window);
        XCloseDisplay(display);
        Some(info)
    }
}

#[cfg(target_os = "linux")]
pub fn get_all_windows() -> Vec<WindowInfo> {
    if crate::linux_desktop::is_wayland() {
        let windows = crate::linux_desktop::all_windows();
        if !windows.is_empty() {
            return windows;
        }
    }
    x11_all_windows()
}

/// Enumerates the window manager's `_NET_CLIENT_LIST`, which is what every EWMH
/// compliant WM publishes for taskbars.
#[cfg(target_os = "linux")]
fn x11_all_windows() -> Vec<WindowInfo> {
    use x11::xlib::*;

    unsafe {
        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return Vec::new();
        }

        let Ok(name) = std::ffi::CString::new("_NET_CLIENT_LIST") else {
            XCloseDisplay(display);
            return Vec::new();
        };
        let atom = XInternAtom(display, name.as_ptr(), 1);
        if atom == 0 {
            XCloseDisplay(display);
            return Vec::new();
        }

        let root = XDefaultRootWindow(display);
        let mut actual_type: Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut data: *mut u8 = std::ptr::null_mut();

        let status = XGetWindowProperty(
            display,
            root,
            atom,
            0,
            4096,
            0,
            0,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut data,
        );

        if status != 0 || data.is_null() {
            XCloseDisplay(display);
            return Vec::new();
        }

        let mut windows = Vec::new();
        if actual_format == 32 {
            let list = data as *const Window;
            for index in 0..nitems as usize {
                let window = std::ptr::read_unaligned(list.add(index));
                let info = x11_window_info(display, window);
                // Windows with neither a title nor a class are not user facing.
                if !info.title.is_empty() || !info.class.is_empty() {
                    windows.push(info);
                }
            }
        }

        XFree(data as *mut _);
        XCloseDisplay(display);
        windows
    }
}
