#[cfg(any(test, target_arch = "wasm32"))]
const DEFAULT_ZOOM: f64 = 1.0;
#[cfg(any(test, target_arch = "wasm32"))]
const MIN_ZOOM: f64 = 0.5;
#[cfg(any(test, target_arch = "wasm32"))]
const MAX_ZOOM: f64 = 2.5;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportState {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<(f64, f64)>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Default for ViewportState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: DEFAULT_ZOOM,
            is_dragging: false,
            last_mouse_pos: None,
        }
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
impl ViewportState {
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    pub fn drag_to(&mut self, x: f64, y: f64) -> bool {
        if !self.is_dragging {
            return false;
        }

        if let Some((last_x, last_y)) = self.last_mouse_pos {
            self.pan_by(x - last_x, y - last_y);
            self.last_mouse_pos = Some((x, y));
            return true;
        }

        self.last_mouse_pos = Some((x, y));
        false
    }

    pub fn pan_by(&mut self, delta_x: f64, delta_y: f64) {
        self.pan_x += delta_x;
        self.pan_y += delta_y;
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn zoom_by(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub fn world_point_at(
        &self,
        screen_x: f64,
        screen_y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> (f64, f64) {
        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;

        (
            (screen_x - center_x - self.pan_x) / self.zoom,
            (screen_y - center_y - self.pan_y) / self.zoom,
        )
    }

    pub fn zoom_at(
        &mut self,
        factor: f64,
        cursor_x: f64,
        cursor_y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) {
        // Preserve the world-space point under the cursor so wheel zoom feels anchored.
        let world_point = self.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);
        self.zoom_by(factor);

        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;
        self.pan_x = cursor_x - center_x - (world_point.0 * self.zoom);
        self.pan_y = cursor_y - center_y - (world_point.1 * self.zoom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_viewport_state_is_centered() {
        let state = ViewportState::default();

        assert_eq!(state.pan_x, 0.0);
        assert_eq!(state.pan_y, 0.0);
        assert_eq!(state.zoom, 1.0);
        assert!(!state.is_dragging);
        assert_eq!(state.last_mouse_pos, None);
    }

    #[test]
    fn dragging_updates_pan_coordinates() {
        let mut state = ViewportState::default();

        state.start_drag(20.0, 40.0);
        assert!(state.drag_to(65.0, 95.0));
        assert_eq!(state.pan_x, 45.0);
        assert_eq!(state.pan_y, 55.0);

        assert!(state.drag_to(80.0, 125.0));
        assert_eq!(state.pan_x, 60.0);
        assert_eq!(state.pan_y, 85.0);
    }

    #[test]
    fn drag_stops_after_release_and_zoom_is_clamped() {
        let mut state = ViewportState::default();

        state.start_drag(0.0, 0.0);
        assert!(state.drag_to(12.0, -18.0));
        state.end_drag();
        assert!(!state.drag_to(30.0, 30.0));
        assert_eq!(state.pan_x, 12.0);
        assert_eq!(state.pan_y, -18.0);

        for _ in 0..12 {
            state.zoom_by(1.3);
        }
        assert_eq!(state.zoom, 2.5);

        for _ in 0..24 {
            state.zoom_by(0.5);
        }
        assert_eq!(state.zoom, 0.5);
    }

    #[test]
    fn panning_by_delta_moves_the_viewport() {
        let mut state = ViewportState::default();

        state.pan_by(24.0, -16.0);
        state.pan_by(-10.0, 6.0);

        assert_eq!(state.pan_x, 14.0);
        assert_eq!(state.pan_y, -10.0);

        state.reset();
        assert_eq!(state, ViewportState::default());
    }

    #[test]
    fn zooming_keeps_the_cursor_world_point_stable() {
        let mut state = ViewportState::default();
        let viewport_width = 800.0;
        let viewport_height = 600.0;
        let cursor_x = 620.0;
        let cursor_y = 420.0;

        let world_before =
            state.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);
        state.zoom_at(1.25, cursor_x, cursor_y, viewport_width, viewport_height);
        let world_after = state.world_point_at(cursor_x, cursor_y, viewport_width, viewport_height);

        assert!((world_before.0 - world_after.0).abs() < 1e-9);
        assert!((world_before.1 - world_after.1).abs() < 1e-9);
        assert!(state.zoom > 1.0);
    }
}
