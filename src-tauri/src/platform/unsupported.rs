use super::{ClipboardBackup, Host, PermissionState};
use ps_core::workflow::WindowTarget;
use ps_core::{PsError, PsResult};

pub struct UnsupportedHost;

fn unsupported() -> PsError {
    PsError::UnsupportedPlatform
}

impl Host for UnsupportedHost {
    fn current_window(&self) -> PsResult<WindowTarget> {
        Err(unsupported())
    }

    fn focus(&self, _target: &WindowTarget) -> PsResult<()> {
        Err(unsupported())
    }

    fn copy_shortcut(&self) -> PsResult<()> {
        Err(unsupported())
    }

    fn paste_shortcut(&self) -> PsResult<()> {
        Err(unsupported())
    }

    fn read_clipboard_text(&self) -> PsResult<Option<String>> {
        Err(unsupported())
    }

    fn write_clipboard_text(&self, _text: &str) -> PsResult<()> {
        Err(unsupported())
    }

    fn snapshot_clipboard(&self) -> PsResult<ClipboardBackup> {
        Err(unsupported())
    }

    fn restore_clipboard(&self, _backup: &ClipboardBackup) -> PsResult<()> {
        Err(unsupported())
    }

    fn accessibility(&self) -> PermissionState {
        PermissionState::Unsupported
    }

    fn open_accessibility_settings(&self) -> PsResult<()> {
        Err(unsupported())
    }
}
