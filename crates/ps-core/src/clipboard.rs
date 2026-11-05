/// Pure clipboard-capture decisions. Platform code performs the actual read and write.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapturePoll {
    /// The clipboard still holds the sentinel or has not updated yet.
    Pending,
    /// A copy happened, but the selection was empty.
    Empty,
    /// The copied selection.
    Ready(String),
}

/// Interpret the clipboard after writing `sentinel` and simulating copy.
///
/// The returned selection keeps its original whitespace. Emptiness uses a trim check
/// so a selection of spaces is treated as no text.
pub fn interpret_clipboard_after_copy(sentinel: &str, current: Option<&str>) -> CapturePoll {
    match current {
        None => CapturePoll::Pending,
        Some(text) if text == sentinel => CapturePoll::Pending,
        Some(text) if text.trim().is_empty() => CapturePoll::Empty,
        Some(text) => CapturePoll::Ready(text.to_string()),
    }
}

pub fn selection_matches(original: &str, current: &str) -> bool {
    normalize_newlines(original) == normalize_newlines(current)
}

pub fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn make_sentinel(operation_id: &str) -> String {
    format!("\u{2063}ps:{operation_id}:\u{2063}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_while_sentinel_remains() {
        let sentinel = make_sentinel("abc");
        assert_eq!(
            interpret_clipboard_after_copy(&sentinel, None),
            CapturePoll::Pending
        );
        assert_eq!(
            interpret_clipboard_after_copy(&sentinel, Some(&sentinel)),
            CapturePoll::Pending
        );
    }

    #[test]
    fn treats_blank_copy_as_empty_and_keeps_real_whitespace() {
        assert_eq!(
            interpret_clipboard_after_copy("sentinel", Some("  \n")),
            CapturePoll::Empty
        );
        assert_eq!(
            interpret_clipboard_after_copy("sentinel", Some("  keep\n")),
            CapturePoll::Ready("  keep\n".into())
        );
    }

    #[test]
    fn newline_style_does_not_count_as_a_changed_selection() {
        assert!(selection_matches("a\r\nb", "a\nb"));
        assert!(!selection_matches("a", "a "));
    }
}
