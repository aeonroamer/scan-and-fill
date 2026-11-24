use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, InternalGetWindowText,
};
use windows::core::PWSTR;

const PATH_CAPACITY: usize = 1024;
const TITLE_CAPACITY: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowInfo {
    pub title: String,
    pub process_id: u32,
    pub exe: Option<PathBuf>,
}

impl WindowInfo {
    pub fn exe_name(&self) -> Option<&str> {
        self.exe.as_ref()?.file_name()?.to_str()
    }
}

pub fn focused_window() -> Option<WindowInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    Some(window_info(hwnd))
}

fn window_info(hwnd: HWND) -> WindowInfo {
    let process_id = process_id(hwnd);
    WindowInfo {
        title: window_title(hwnd),
        process_id,
        exe: process_path(process_id),
    }
}

fn window_title(hwnd: HWND) -> String {
    let mut buf = vec![0u16; TITLE_CAPACITY];
    let written = unsafe { InternalGetWindowText(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..written.max(0) as usize])
}

fn process_id(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
    pid
}

fn process_path(pid: u32) -> Option<PathBuf> {
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;

    let mut buf = vec![0u16; PATH_CAPACITY];
    let mut len = buf.len() as u32;
    let result = unsafe {
        QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
    };
    let _ = unsafe { CloseHandle(handle) };

    result.ok()?;
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])))
}
