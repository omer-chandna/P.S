use super::clipboard_io;
use super::{ClipboardBackup, Host, PermissionState};
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::CFStringRef;
use ps_core::workflow::WindowTarget;
use ps_core::{PsError, PsResult};
use std::path::{Component, Path};
use std::process::Command;

pub struct MacHost;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

pub(crate) fn accessibility_list_name() -> &'static str {
    list_name_for_path(std::env::current_exe().ok().as_deref())
}

fn list_name_for_path(path: Option<&Path>) -> &'static str {
    let bundled = path.is_some_and(|path| {
        path.components().any(|component| match component {
            Component::Normal(name) => name.to_string_lossy().ends_with(".app"),
            _ => false,
        })
    });
    if bundled {
        "PS"
    } else {
        "postscript"
    }
}

/// Registers this process in the Accessibility list and shows the system dialog
/// when it is not already trusted. A plain `AXIsProcessTrusted` check does neither.
fn prompt_for_accessibility() {
    use core_foundation::string::CFString;

    let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe {
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
    }
}

impl Host for MacHost {
    fn current_window(&self) -> PsResult<WindowTarget> {
        let output = run_osascript(
            r#"tell application "System Events"
set frontProc to first application process whose frontmost is true
set appName to name of frontProc
set procId to unix id of frontProc
set winTitle to ""
try
  set winTitle to name of front window of frontProc
end try
return appName & linefeed & procId & linefeed & winTitle
end tell"#,
        )?;
        parse_frontmost(&output)
    }

    fn focus(&self, target: &WindowTarget) -> PsResult<()> {
        let script = format!(
            "tell application \"System Events\" to set frontmost of (first application process whose unix id is {}) to true",
            target.pid
        );
        run_osascript(&script)?;
        Ok(())
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
        if unsafe { AXIsProcessTrusted() } != 0 {
            PermissionState::Granted
        } else {
            PermissionState::Missing
        }
    }

    fn open_accessibility_settings(&self) -> PsResult<()> {
        prompt_for_accessibility();
        let urls = [
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility",
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        ];
        for url in urls {
            if Command::new("open")
                .arg(url)
                .status()
                .is_ok_and(|status| status.success())
            {
                return Ok(());
            }
        }
        Err(PsError::MissingPermissions {
            permission: "Accessibility".into(),
        })
    }
}

fn run_osascript(script: &str) -> PsResult<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|_| PsError::MissingPermissions {
            permission: "Accessibility".into(),
        })?;
    if !output.status.success() {
        return Err(PsError::MissingPermissions {
            permission: "Accessibility".into(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_frontmost(output: &str) -> PsResult<WindowTarget> {
    let mut lines = output.lines();
    let app_name = lines.next().unwrap_or("").trim().to_string();
    let pid: u32 = lines.next().unwrap_or("0").trim().parse().unwrap_or(0);
    let title = lines.next().unwrap_or("").trim().to_string();
    if pid == 0 || app_name.is_empty() {
        return Err(PsError::FocusChanged);
    }
    Ok(WindowTarget {
        window_title: if title.is_empty() {
            app_name.clone()
        } else {
            format!("{app_name} — {title}")
        },
        app_name: format!("pid:{pid}"),
        pid,
        native_id: i64::from(pid),
    })
}

#[cfg(test)]
mod tests {
    use super::list_name_for_path;
    use std::path::Path;

    #[test]
    fn bundled_app_is_listed_as_ps() {
        let path = Path::new("/Applications/PS.app/Contents/MacOS/postscript");
        assert_eq!(list_name_for_path(Some(path)), "PS");
    }

    #[test]
    fn dev_binary_is_listed_as_postscript() {
        let path = Path::new("/Users/me/P.S/target/debug/postscript");
        assert_eq!(list_name_for_path(Some(path)), "postscript");
    }
}
