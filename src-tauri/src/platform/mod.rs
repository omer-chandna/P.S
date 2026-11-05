#![cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod clipboard_io;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod input;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use ps_core::clipboard::{interpret_clipboard_after_copy, make_sentinel, CapturePoll};
use ps_core::workflow::WindowTarget;
use ps_core::{PsError, PsResult};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OwnedImage {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ClipboardBackup {
    pub text: Option<String>,
    pub image: Option<OwnedImage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Missing,
    /// Used when capture is not implemented for the host.
    #[allow(dead_code)]
    Unsupported,
}

pub trait Host: Send + Sync {
    fn current_window(&self) -> PsResult<WindowTarget>;
    fn focus(&self, target: &WindowTarget) -> PsResult<()>;
    fn copy_shortcut(&self) -> PsResult<()>;
    fn paste_shortcut(&self) -> PsResult<()>;
    fn read_clipboard_text(&self) -> PsResult<Option<String>>;
    fn write_clipboard_text(&self, text: &str) -> PsResult<()>;
    fn snapshot_clipboard(&self) -> PsResult<ClipboardBackup>;
    fn restore_clipboard(&self, backup: &ClipboardBackup) -> PsResult<()>;
    fn accessibility(&self) -> PermissionState;
    fn open_accessibility_settings(&self) -> PsResult<()>;
}

pub fn enrich_hardware(profile: &mut ps_core::hardware::HardwareProfile) {
    #[cfg(target_os = "windows")]
    windows::enrich_hardware(profile);
    #[cfg(not(target_os = "windows"))]
    let _ = profile;
}

pub fn host() -> &'static dyn Host {
    #[cfg(target_os = "macos")]
    {
        &macos::MacHost
    }
    #[cfg(target_os = "windows")]
    {
        &windows::WindowsHost
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        &unsupported::UnsupportedHost
    }
}

pub fn capture_selection(host: &dyn Host) -> PsResult<String> {
    let backup = host.snapshot_clipboard()?;
    let sentinel = make_sentinel(&Uuid::new_v4().to_string());
    host.write_clipboard_text(&sentinel)?;
    if let Err(error) = host.copy_shortcut() {
        let _ = host.restore_clipboard(&backup);
        return Err(error);
    }

    let started = Instant::now();
    let mut captured = None;
    while started.elapsed() < Duration::from_millis(450) {
        std::thread::sleep(Duration::from_millis(30));
        let current = host.read_clipboard_text().ok().flatten();
        match interpret_clipboard_after_copy(&sentinel, current.as_deref()) {
            CapturePoll::Pending => continue,
            CapturePoll::Empty => break,
            CapturePoll::Ready(text) => {
                captured = Some(text);
                break;
            }
        }
    }

    host.restore_clipboard(&backup)?;
    match captured {
        Some(text) if !text.trim().is_empty() => Ok(text),
        _ => Err(PsError::NoSelection),
    }
}

pub fn paste_text(host: &dyn Host, text: &str) -> PsResult<()> {
    let backup = host.snapshot_clipboard()?;
    host.write_clipboard_text(text)?;
    let pasted = host.paste_shortcut();
    std::thread::sleep(Duration::from_millis(80));
    let restored = host.restore_clipboard(&backup);
    pasted?;
    restored.map_err(|_| PsError::ClipboardUnavailable {
        detail: "The prompt was pasted, but the previous clipboard could not be restored.".into(),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeHost {
        clipboard: Mutex<String>,
        backup_text: Mutex<Option<String>>,
        selection: String,
        copied: Mutex<bool>,
    }

    impl Host for FakeHost {
        fn current_window(&self) -> PsResult<WindowTarget> {
            Ok(WindowTarget {
                app_name: "Editor".into(),
                window_title: "note".into(),
                pid: 7,
                native_id: 7,
            })
        }

        fn focus(&self, _target: &WindowTarget) -> PsResult<()> {
            Ok(())
        }

        fn copy_shortcut(&self) -> PsResult<()> {
            *self.copied.lock().unwrap() = true;
            *self.clipboard.lock().unwrap() = self.selection.clone();
            Ok(())
        }

        fn paste_shortcut(&self) -> PsResult<()> {
            Ok(())
        }

        fn read_clipboard_text(&self) -> PsResult<Option<String>> {
            Ok(Some(self.clipboard.lock().unwrap().clone()))
        }

        fn write_clipboard_text(&self, text: &str) -> PsResult<()> {
            *self.clipboard.lock().unwrap() = text.to_string();
            Ok(())
        }

        fn snapshot_clipboard(&self) -> PsResult<ClipboardBackup> {
            let text = self.clipboard.lock().unwrap().clone();
            *self.backup_text.lock().unwrap() = Some(text.clone());
            Ok(ClipboardBackup {
                text: Some(text),
                image: None,
            })
        }

        fn restore_clipboard(&self, backup: &ClipboardBackup) -> PsResult<()> {
            if let Some(text) = &backup.text {
                *self.clipboard.lock().unwrap() = text.clone();
            }
            Ok(())
        }

        fn accessibility(&self) -> PermissionState {
            PermissionState::Granted
        }

        fn open_accessibility_settings(&self) -> PsResult<()> {
            Ok(())
        }
    }

    #[test]
    fn capture_reads_the_selection_and_restores_the_clipboard() {
        let host = FakeHost {
            clipboard: Mutex::new("previous clip".into()),
            backup_text: Mutex::new(None),
            selection: "fix login it breaks on refresh".into(),
            copied: Mutex::new(false),
        };
        let captured = capture_selection(&host).unwrap();
        assert_eq!(captured, "fix login it breaks on refresh");
        assert_eq!(host.clipboard.lock().unwrap().as_str(), "previous clip");
        assert!(*host.copied.lock().unwrap());
    }

    #[test]
    fn capture_reports_when_nothing_is_selected() {
        let host = FakeHost {
            clipboard: Mutex::new("previous clip".into()),
            backup_text: Mutex::new(None),
            selection: "   ".into(),
            copied: Mutex::new(false),
        };
        // Blank selection overwrites the sentinel with whitespace, which is empty.
        let error = capture_selection(&host).unwrap_err();
        assert!(matches!(error, PsError::NoSelection));
        assert_eq!(host.clipboard.lock().unwrap().as_str(), "previous clip");
    }
}
