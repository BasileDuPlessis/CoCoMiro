//! # Viewport Module
//!
//! This module manages the viewport/camera system for the CoCoMiro infinite canvas.
//!
//! ## Architecture
//!
//! The viewport represents the visible portion of the infinite 2D world. It handles:
//! - Pan operations (translating the view)
//! - Zoom operations (scaling the view)
//! - Coordinate transformations between screen and world space
//! - Drag operations for panning
//!
//! ## Coordinate Systems
//!
//! - **Screen coordinates**: Pixel positions relative to the canvas element (0,0 at top-left)
//! - **World coordinates**: Infinite 2D space where content exists, independent of screen
//!
//! The viewport transforms between these coordinate systems using pan and zoom parameters.
//!
//! ## Zoom Constraints
//!
//! Zoom is constrained between `MIN_ZOOM` (0.5) and `MAX_ZOOM` (2.5) to maintain
//! usability and prevent extreme scaling that could cause rendering issues.
//!
//! ## Pan and Zoom Interaction
//!
//! - **Panning**: Moves the viewport over the world space
//! - **Zooming**: Scales the view while keeping a point stationary under the cursor
//! - **Dragging**: Smooth viewport panning via mouse drag operations

#[cfg(any(test, target_arch = "wasm32"))]
const DEFAULT_ZOOM: f64 = 1.0;
#[cfg(any(test, target_arch = "wasm32"))]
const MIN_ZOOM: f64 = 0.5;
#[cfg(any(test, target_arch = "wasm32"))]
const MAX_ZOOM: f64 = 2.5;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
/// Represents the current viewport/camera state for the infinite canvas.
///
/// The viewport defines which portion of the infinite world is currently visible
/// on screen, including position (pan), scale (zoom), and interaction state (dragging).
pub struct ViewportState {
    /// Horizontal offset of the viewport center in world coordinates
    pub pan_x: f64,
    /// Vertical offset of the viewport center in world coordinates
    pub pan_y: f64,
    /// Current zoom level (scale factor), constrained between MIN_ZOOM and MAX_ZOOM
    pub zoom: f64,
    /// Whether the viewport is currently being dragged (panned)
    pub is_dragging: bool,
    /// Last recorded mouse position during drag operations
    pub last_mouse_pos: Option<(f64, f64)>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl ViewportState {
    /// Initiates a viewport drag (pan) operation.
    ///
    /// This method sets the dragging state and records the initial mouse
    /// position to establish the drag offset for smooth panning.
    ///
    /// # Arguments
    /// * `x` - Initial X-coordinate of the mouse cursor in screen pixels
    /// * `y` - Initial Y-coordinate of the mouse cursor in screen pixels
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    /// Updates the viewport position during a drag operation.
    ///
    /// This method pans the viewport to follow the mouse cursor, maintaining
    /// the offset established when dragging started. Only has effect if
    /// a drag operation is currently active.
    ///
    /// # Arguments
    /// * `x` - Current X-coordinate of the mouse cursor in screen pixels
    /// * `y` - Current Y-coordinate of the mouse cursor in screen pixels
    ///
    /// # Returns
    /// `true` if the viewport position was updated, `false` if not dragging
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

    /// Terminates the current viewport drag operation.
    ///
    /// This method resets the dragging state and clears the mouse position tracking,
    /// allowing new drag operations to start cleanly.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_mouse_pos = None;
    }

    /// Pans the viewport by the specified delta amounts.
    ///
    /// This method translates the viewport position in world coordinates,
    /// effectively moving the view over the infinite canvas.
    ///
    /// # Arguments
    /// * `delta_x` - Amount to pan horizontally in screen pixels
    /// * `delta_y` - Amount to pan vertically in screen pixels
    pub fn pan_by(&mut self, delta_x: f64, delta_y: f64) {
        self.pan_x += delta_x;
        self.pan_y += delta_y;
    }

    /// Resets the viewport to its default state.
    ///
    /// This method restores the viewport to the origin (0,0) with default zoom,
    /// clearing any pan or zoom that has been applied. Equivalent to creating
    /// a new default ViewportState.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Applies a zoom factor to the viewport, centered on the current center.
    ///
    /// This method scales the viewport by multiplying the current zoom by the factor,
    /// clamping the result to stay within the allowed zoom range (MIN_ZOOM to MAX_ZOOM).
    ///
    /// # Arguments
    /// * `factor` - Zoom multiplier (e.g., 1.1 for 10% zoom in, 0.9 for 10% zoom out)
    pub fn zoom_by(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Converts a screen coordinate to world coordinates.
    ///
    /// This method transforms a point from screen pixel coordinates to world coordinates,
    /// taking into account the current pan and zoom of the viewport.
    ///
    /// # Arguments
    /// * `screen_x` - X-coordinate in screen pixels (relative to canvas top-left)
    /// * `screen_y` - Y-coordinate in screen pixels (relative to canvas top-left)
    /// * `viewport_width` - Width of the viewport in screen pixels
    /// * `viewport_height` - Height of the viewport in screen pixels
    ///
    /// # Returns
    /// A tuple `(world_x, world_y)` representing the point in world coordinates
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

    /// Applies a zoom factor centered on a specific screen point.
    ///
    /// This method zooms the viewport while keeping the world point under the cursor
    /// stationary. It first calculates the world coordinates under the cursor, applies
    /// the zoom, then adjusts the pan so the cursor remains over the same world point.
    ///
    /// This creates the intuitive "zoom toward cursor" behavior where the point
    /// under the mouse stays fixed during zoom operations.
    ///
    /// # Arguments
    /// * `factor` - Zoom multiplier
    /// * `cursor_x` - X-coordinate of the cursor in screen pixels
    /// * `cursor_y` - Y-coordinate of the cursor in screen pixels
    /// * `viewport_width` - Width of the viewport in screen pixels
    /// * `viewport_height` - Height of the viewport in screen pixels
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
