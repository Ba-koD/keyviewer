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
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
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

        // Get process name
        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let mut process_name = String::new();
        if process_id != 0 {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id) {
                let mut name_buf = [0u16; 512];
                let len = GetModuleBaseNameW(handle, None, &mut name_buf);
                if len > 0 {
                    process_name = String::from_utf16_lossy(&name_buf[..len as usize]);
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
    use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
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
                    // Get process name
                    let mut process_id = 0u32;
                    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

                    let mut process_name = String::new();
                    if process_id != 0 {
                        if let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id) {
                            let mut name_buf = [0u16; 512];
                            let len = GetModuleBaseNameW(handle, None, &mut name_buf);
                            if len > 0 {
                                process_name = String::from_utf16_lossy(&name_buf[..len as usize]);
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
    use cocoa::appkit::NSRunningApplication;
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use core_foundation::string::CFString;
    use core_graphics::window::{kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo};

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        
        let app: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let active_app: id = msg_send![app, frontmostApplication];
        
        if active_app == nil {
            return None;
        }

        let bundle_id: id = msg_send![active_app, bundleIdentifier];
        let process_name: id = msg_send![active_app, localizedName];
        let pid: i32 = msg_send![active_app, processIdentifier];

        let process_str = if process_name != nil {
            let s: *const i8 = msg_send![process_name, UTF8String];
            std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
        } else {
            String::new()
        };

        // Try to get window title from CGWindowList
        let mut title = String::new();
        if let Some(window_list) = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, 0) {
            // Find window with matching PID
            // This is simplified - full implementation would require more Core Foundation work
        }

        Some(WindowInfo {
            hwnd: format!("{}", pid),
            title,
            process: process_str,
            class: String::new(),
        })
    }
}

#[cfg(target_os = "macos")]
pub fn get_all_windows() -> Vec<WindowInfo> {
    // Simplified - full implementation requires more Core Foundation work
    Vec::new()
}

#[cfg(target_os = "linux")]
pub fn get_foreground_window() -> Option<WindowInfo> {
    use x11::xlib::*;
    use std::ffi::CString;
    use std::ptr;

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
            std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned()
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
            std::ffi::CStr::from_ptr(class_hint.res_class).to_string_lossy().into_owned()
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

