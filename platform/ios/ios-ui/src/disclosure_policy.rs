use core::cell::Cell;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::Frame;

pub(crate) const DISCLOSURE_HEADER_HEIGHT: f64 = 44.0;

pub(crate) fn content_height(frame: Frame) -> Result<f64> {
    if frame.height() < DISCLOSURE_HEADER_HEIGHT {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(frame.height() - DISCLOSURE_HEADER_HEIGHT)
}

pub(crate) fn update_expanded(expanded: &Cell<bool>, next: bool) -> bool {
    expanded.replace(next) != next
}

pub(crate) fn visible_height(expanded: bool, total_height: f64) -> f64 {
    if expanded {
        total_height
    } else {
        DISCLOSURE_HEADER_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use framework_core::ErrorKind;
    use framework_ui::Frame;

    use super::{DISCLOSURE_HEADER_HEIGHT, content_height, update_expanded, visible_height};

    #[test]
    fn disclosure_frame_rejects_less_than_header_height() {
        let frame = Frame::new(0.0, 0.0, 240.0, DISCLOSURE_HEADER_HEIGHT - 1.0).unwrap();
        assert_eq!(
            content_height(frame).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn disclosure_frame_preserves_expanded_content_height() {
        let frame = Frame::new(0.0, 0.0, 240.0, DISCLOSURE_HEADER_HEIGHT + 120.0).unwrap();
        assert_eq!(content_height(frame), Ok(120.0));
    }

    #[test]
    fn disclosure_frame_accepts_header_only_boundary() {
        let frame = Frame::new(0.0, 0.0, 240.0, DISCLOSURE_HEADER_HEIGHT).unwrap();
        assert_eq!(content_height(frame), Ok(0.0));
    }

    #[test]
    fn expanded_updates_are_idempotent() {
        let expanded = Cell::new(false);
        assert!(!update_expanded(&expanded, false));
        assert!(update_expanded(&expanded, true));
        assert!(expanded.get());
        assert!(!update_expanded(&expanded, true));
        assert!(update_expanded(&expanded, false));
        assert!(!expanded.get());
    }

    #[test]
    fn visible_height_collapses_to_header_and_restores_expanded_frame() {
        assert_eq!(visible_height(false, 164.0), DISCLOSURE_HEADER_HEIGHT);
        assert_eq!(visible_height(true, 164.0), 164.0);
    }
}
