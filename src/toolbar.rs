#[cfg(any(test, target_arch = "wasm32"))]
const TOOLBAR_DEFAULT_X: f64 = 18.0;
#[cfg(any(test, target_arch = "wasm32"))]
const TOOLBAR_DEFAULT_Y: f64 = 18.0;
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) const TOOLBAR_EDGE_PADDING: f64 = 12.0;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingToolbarState {
    pub x: f64,
    pub y: f64,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<(f64, f64)>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Default for FloatingToolbarState {
    fn default() -> Self {
        Self {
            x: TOOLBAR_DEFAULT_X,
            y: TOOLBAR_DEFAULT_Y,
            is_dragging: false,
            last_mouse_pos: None,
        }
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
impl FloatingToolbarState {
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    pub fn drag_to(&mut self, x: f64, y: f64) -> bool {
        if !self.is_dragging {
            return false;
        }

        if let Some((last_x, last_y)) = self.last_mouse_pos {
            self.x += x - last_x;
            self.y += y - last_y;
            self.last_mouse_pos = Some((x, y));
            return true;
        }

        self.last_mouse_pos = Some((x, y));
        false
    }

    pub fn clamp_within(&mut self, max_x: f64, max_y: f64) {
        self.x = self
            .x
            .clamp(TOOLBAR_EDGE_PADDING, max_x.max(TOOLBAR_EDGE_PADDING));
        self.y = self
            .y
            .clamp(TOOLBAR_EDGE_PADDING, max_y.max(TOOLBAR_EDGE_PADDING));
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_toolbar_dragging_updates_its_position() {
        let mut toolbar = FloatingToolbarState::default();

        assert_eq!(toolbar.x, TOOLBAR_DEFAULT_X);
        assert_eq!(toolbar.y, TOOLBAR_DEFAULT_Y);
        assert!(!toolbar.is_dragging);

        toolbar.start_drag(40.0, 30.0);
        assert!(toolbar.drag_to(70.0, 85.0));
        assert_eq!(toolbar.x, TOOLBAR_DEFAULT_X + 30.0);
        assert_eq!(toolbar.y, TOOLBAR_DEFAULT_Y + 55.0);

        toolbar.end_drag();
        assert!(!toolbar.drag_to(90.0, 100.0));
    }

    #[test]
    fn floating_toolbar_stays_within_bounds_when_clamped() {
        let mut toolbar = FloatingToolbarState::default();

        toolbar.x = -100.0;
        toolbar.y = 400.0;
        toolbar.clamp_within(72.0, 64.0);

        assert_eq!(toolbar.x, 12.0);
        assert_eq!(toolbar.y, 64.0);
    }
}
