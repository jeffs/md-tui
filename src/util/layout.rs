//! Placement of rendered content within the terminal.

use super::general::Centering;

/// Compute the x-offset at which content should begin, based on
/// the terminal width, configured content width, and alignment.
pub fn content_x(terminal_width: u16, config_width: u16, centering: &Centering) -> u16 {
    match centering {
        Centering::Left => 2,
        Centering::Center => {
            let x = (terminal_width / 2).saturating_sub(config_width / 2);
            if x > 2 { x } else { 2 }
        }
        Centering::Right => {
            let x = terminal_width.saturating_sub(config_width + 2);
            if x > 2 { x } else { 2 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_x_left_always_2() {
        assert_eq!(content_x(120, 80, &Centering::Left), 2);
        assert_eq!(content_x(40, 80, &Centering::Left), 2);
        assert_eq!(content_x(80, 80, &Centering::Left), 2);
    }

    #[test]
    fn content_x_center_wide_terminal() {
        // terminal=120, content=80 → (120/2) - (80/2) = 60 - 40 = 20
        assert_eq!(content_x(120, 80, &Centering::Center), 20);
    }

    #[test]
    fn content_x_center_exact_fit() {
        // terminal=80, content=80 → (80/2) - (80/2) = 0, clamped to 2
        assert_eq!(content_x(80, 80, &Centering::Center), 2);
    }

    #[test]
    fn content_x_center_narrow_terminal() {
        // terminal=40, content=80 → (40/2) - (80/2) = 20 - 40 = 0 (saturating), clamped to 2
        assert_eq!(content_x(40, 80, &Centering::Center), 2);
    }

    #[test]
    fn content_x_right_wide_terminal() {
        // terminal=120, content=80 → 120 - (80 + 2) = 38
        assert_eq!(content_x(120, 80, &Centering::Right), 38);
    }

    #[test]
    fn content_x_right_exact_fit() {
        // terminal=82, content=80 → 82 - 82 = 0, clamped to 2
        assert_eq!(content_x(82, 80, &Centering::Right), 2);
    }

    #[test]
    fn content_x_right_narrow_terminal() {
        // terminal=40, content=80 → 40 - 82 = 0 (saturating), clamped to 2
        assert_eq!(content_x(40, 80, &Centering::Right), 2);
    }
}
