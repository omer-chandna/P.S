use super::{ClipboardBackup, OwnedImage};
use ps_core::{PsError, PsResult};

pub fn snapshot() -> PsResult<ClipboardBackup> {
    let mut clipboard = arboard::Clipboard::new().map_err(|_| PsError::ClipboardUnavailable {
        detail: "The clipboard is unavailable.".into(),
    })?;
    let text = clipboard.get_text().ok();
    let image = clipboard.get_image().ok().map(|image| OwnedImage {
        width: image.width,
        height: image.height,
        bytes: image.bytes.into_owned(),
    });
    Ok(ClipboardBackup { text, image })
}

pub fn read_text() -> PsResult<Option<String>> {
    let mut clipboard = arboard::Clipboard::new().map_err(|_| PsError::ClipboardUnavailable {
        detail: "The clipboard is unavailable.".into(),
    })?;
    match clipboard.get_text() {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None),
    }
}

pub fn write_text(text: &str) -> PsResult<()> {
    let mut clipboard = arboard::Clipboard::new().map_err(|_| PsError::ClipboardUnavailable {
        detail: "The clipboard is unavailable.".into(),
    })?;
    clipboard
        .set_text(text)
        .map_err(|_| PsError::ClipboardUnavailable {
            detail: "The clipboard rejected the text.".into(),
        })
}

pub fn restore(backup: &ClipboardBackup) -> PsResult<()> {
    let mut clipboard = arboard::Clipboard::new().map_err(|_| PsError::ClipboardUnavailable {
        detail: "The clipboard is unavailable.".into(),
    })?;
    if let Some(image) = &backup.image {
        if backup.text.is_none() {
            let image_data = arboard::ImageData {
                width: image.width,
                height: image.height,
                bytes: std::borrow::Cow::Borrowed(&image.bytes),
            };
            return clipboard
                .set_image(image_data)
                .map_err(|_| PsError::ClipboardUnavailable {
                    detail: "The previous image clipboard could not be restored.".into(),
                });
        }
    }
    if let Some(text) = &backup.text {
        return clipboard
            .set_text(text)
            .map_err(|_| PsError::ClipboardUnavailable {
                detail: "The previous clipboard text could not be restored.".into(),
            });
    }
    clipboard
        .clear()
        .map_err(|_| PsError::ClipboardUnavailable {
            detail: "The clipboard could not be cleared.".into(),
        })
}
