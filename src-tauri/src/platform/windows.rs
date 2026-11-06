use super::clipboard_io;
use super::{ClipboardBackup, Host, PermissionState};
use ps_core::hardware::HardwareProfile;
use ps_core::workflow::WindowTarget;
use ps_core::{PsError, PsResult};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, SetForegroundWindow,
};

pub struct WindowsHost;

impl Host for WindowsHost {
    fn current_window(&self) -> PsResult<WindowTarget> {
        unsafe { foreground_target() }
    }

    fn focus(&self, target: &WindowTarget) -> PsResult<()> {
        let hwnd = HWND(target.native_id as *mut std::ffi::c_void);
        let focused = unsafe { SetForegroundWindow(hwnd) };
        if focused.as_bool() {
            Ok(())
        } else {
            Err(PsError::FocusChanged)
        }
    }

    fn copy_shortcut(&self) -> PsResult<()> {
        super::input::copy_shortcut()
    }

    fn paste_shortcut(&self) -> PsResult<()> {
        super::input::paste_shortcut()
    }

    fn read_clipboard_text(&self) -> PsResult<Option<String>> {
        clipboard_io::read_text()
    }

    fn write_clipboard_text(&self, text: &str) -> PsResult<()> {
        clipboard_io::write_text(text)
    }

    fn snapshot_clipboard(&self) -> PsResult<ClipboardBackup> {
        clipboard_io::snapshot()
    }

    fn restore_clipboard(&self, backup: &ClipboardBackup) -> PsResult<()> {
        clipboard_io::restore(backup)
    }

    fn accessibility(&self) -> PermissionState {
        PermissionState::Granted
    }

    fn open_accessibility_settings(&self) -> PsResult<()> {
        Err(PsError::UnsupportedPlatform)
    }
}

unsafe fn foreground_target() -> PsResult<WindowTarget> {
    let hwnd = GetForegroundWindow();
    if hwnd.0.is_null() {
        return Err(PsError::FocusChanged);
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(std::ptr::addr_of_mut!(pid)));
    if pid == 0 {
        return Err(PsError::FocusChanged);
    }
    let title = window_text(hwnd);
    let process = process_name(pid).unwrap_or_else(|| "Windows application".into());
    Ok(WindowTarget {
        window_title: if title.is_empty() {
            process
        } else {
            format!("{process} — {title}")
        },
        app_name: format!("pid:{pid}"),
        pid,
        native_id: hwnd.0 as isize as i64,
    })
}

unsafe fn window_text(hwnd: HWND) -> String {
    let mut buffer = [0u16; 512];
    let length = GetWindowTextW(hwnd, &mut buffer);
    if length <= 0 {
        return String::new();
    }
    OsString::from_wide(&buffer[..length as usize])
        .to_string_lossy()
        .trim()
        .to_string()
}

unsafe fn process_name(pid: u32) -> Option<String> {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
    let mut buffer = [0u16; 512];
    let mut size = buffer.len() as u32;
    let ok = QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_WIN32,
        windows::core::PWSTR(buffer.as_mut_ptr()),
        &mut size,
    );
    let _ = windows::Win32::Foundation::CloseHandle(handle);
    if ok.is_err() || size == 0 {
        return None;
    }
    let full = OsString::from_wide(&buffer[..size as usize]);
    let full = full.to_string_lossy().trim_end_matches('\0').to_string();
    Some(
        full.rsplit(['\\', '/'])
            .next()
            .unwrap_or("Windows application")
            .to_string(),
    )
}

pub fn enrich_hardware(profile: &mut HardwareProfile) {
    if profile.gpu_name.is_some() {
        return;
    }
    profile.detection_notes.push(
        "Discrete GPU memory was not read. Windows reports video memory inconsistently, so choose a model from system RAM or pick one manually.".into(),
    );
}
