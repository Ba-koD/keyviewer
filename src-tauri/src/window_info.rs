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

    const kCGWindowListOptionAll: u32 = 0;
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
                    .and_then(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        Some(s.to_string())
                    })
                    .unwrap_or_default();

                let window_name = window_info
                    .find(window_name_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        Some(s.to_string())
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
                if window_name.is_empty() && !owner_name.is_empty() {
                    if !TITLE_WARNING_SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        eprintln!("\n⚠️  [macOS] Window titles are EMPTY!");
                        eprintln!("⚠️  Screen Recording permission is probably missing.");
                        eprintln!(
                            "⚠️  Fix: System Settings > Privacy & Security > Screen Recording"
                        );
                        eprintln!("⚠️  Enable 'KeyQueueViewer', then QUIT (Cmd+Q) and restart.\n");
                    }
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
                    .and_then(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        Some(s.to_string())
                    })
                    .unwrap_or_default();

                let window_name = window_info
                    .find(window_name_key.as_concrete_TypeRef())
                    .and_then(|v| {
                        let s: CFString =
                            CFString::wrap_under_get_rule(v.as_CFTypeRef() as CFStringRef);
                        Some(s.to_string())
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

#[cfg(target_os = "linux")]
pub fn get_foreground_window() -> Option<WindowInfo> {
    use std::ffi::CString;
    use std::ptr;
    use x11::xlib::*;

    unsafe {
        let display = XOpenDisplay(ptr::null());
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

        // Get window title
        let mut name: *mut i8 = ptr::null_mut();
        XFetchName(display, focus_window, &mut name);
        let title = if !name.is_null() {
            std::ffi::CStr::from_ptr(name)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };

        if !name.is_null() {
            XFree(name as *mut _);
        }

        // Get window class
        let mut class_hint = XClassHint {
            res_name: ptr::null_mut(),
            res_class: ptr::null_mut(),
        };
        XGetClassHint(display, focus_window, &mut class_hint);

        let class_name = if !class_hint.res_class.is_null() {
            std::ffi::CStr::from_ptr(class_hint.res_class)
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };

        if !class_hint.res_name.is_null() {
            XFree(class_hint.res_name as *mut _);
        }
        if !class_hint.res_class.is_null() {
            XFree(class_hint.res_class as *mut _);
        }

        XCloseDisplay(display);

        Some(WindowInfo {
            hwnd: format!("{}", focus_window),
            title,
            process: String::new(), // Getting process name on Linux requires /proc parsing
            class: class_name,
        })
    }
}

#[cfg(target_os = "linux")]
pub fn get_all_windows() -> Vec<WindowInfo> {
    // Simplified - full implementation requires X11 window enumeration
    Vec::new()
}
