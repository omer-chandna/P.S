use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use ps_core::{PsError, PsResult};

pub fn chord(modifier: Key, key: char) -> PsResult<()> {
    let mut enigo = Enigo::new(&Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    })
    .map_err(|_| PsError::ClipboardUnavailable {
        detail: "Keyboard control is unavailable.".into(),
    })?;
    enigo
        .key(modifier, Direction::Press)
        .map_err(|_| keyboard_error())?;
    let typed = enigo.key(Key::Unicode(key), Direction::Click);
    let released = enigo.key(modifier, Direction::Release);
    typed.map_err(|_| keyboard_error())?;
    released.map_err(|_| keyboard_error())?;
    Ok(())
}

fn keyboard_error() -> PsError {
    PsError::ClipboardUnavailable {
        detail: "The keyboard shortcut could not be simulated.".into(),
    }
}

#[cfg(target_os = "macos")]
pub fn copy_shortcut() -> PsResult<()> {
    chord(Key::Meta, 'c')
}

#[cfg(target_os = "macos")]
pub fn paste_shortcut() -> PsResult<()> {
    chord(Key::Meta, 'v')
}

#[cfg(target_os = "windows")]
pub fn copy_shortcut() -> PsResult<()> {
    chord(Key::Control, 'c')
}

#[cfg(target_os = "windows")]
pub fn paste_shortcut() -> PsResult<()> {
    chord(Key::Control, 'v')
}
