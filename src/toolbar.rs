//! # Toolbar Module
//!
//! This module manages the floating toolbar UI component in the CoCoMiro application.
//!
//! ## Architecture
//!
//! The toolbar provides quick access to common actions like adding new sticky notes.
//! It is implemented as a floating element that can be dragged around the screen
//! to avoid obstructing the canvas content.
//!
//! ## Positioning
//!
//! The toolbar uses screen coordinates (not world coordinates) since it floats
//! above the canvas viewport. It maintains its position relative to the browser
//! window and can be repositioned by dragging its handle.
//!
//! ## Drag Operations
//!
//! Similar to sticky notes, the toolbar supports smooth dragging with offset
//! tracking. When dragging starts, the offset between the mouse cursor and
//! toolbar position is recorded for consistent movement.
//!
//! ## Constants
//!
//! - `TOOLBAR_DEFAULT_X`: Default horizontal position from top-left corner
//! - `TOOLBAR_DEFAULT_Y`: Default vertical position from top-left corner
//! - `TOOLBAR_EDGE_PADDING`: Minimum distance from screen edges

#[cfg(any(test, target_arch = "wasm32"))]
const TOOLBAR_DEFAULT_X: f64 = 18.0;
#[cfg(any(test, target_arch = "wasm32"))]
const TOOLBAR_DEFAULT_Y: f64 = 18.0;
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) const TOOLBAR_EDGE_PADDING: f64 = 12.0;

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
/// Represents the state and position of the floating toolbar.
///
/// The toolbar is positioned in screen coordinates and can be dragged
/// around the viewport. It maintains its own dragging state separate
/// from canvas interactions.
pub struct FloatingToolbarState {
    /// X-coordinate of the toolbar's top-left corner in screen pixels
    pub x: f64,
    /// Y-coordinate of the toolbar's top-left corner in screen pixels
    pub y: f64,
    /// Whether the toolbar is currently being dragged
    pub is_dragging: bool,
    /// Last recorded mouse position during drag operations
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
    /// Initiates a drag operation for the toolbar.
    ///
    /// This method sets the dragging state and records the initial mouse
    /// position to establish the drag offset for smooth movement.
    ///
    /// # Arguments
    /// * `x` - Initial X-coordinate of the mouse cursor in screen pixels
    /// * `y` - Initial Y-coordinate of the mouse cursor in screen pixels
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.is_dragging = true;
        self.last_mouse_pos = Some((x, y));
    }

    /// Updates the toolbar position during a drag operation.
    ///
    /// This method moves the toolbar to follow the mouse cursor, maintaining
    /// the offset established when dragging started. Only has effect if
    /// a drag operation is currently active.
    ///
    /// # Arguments
    /// * `x` - Current X-coordinate of the mouse cursor in screen pixels
    /// * `y` - Current Y-coordinate of the mouse cursor in screen pixels
    ///
    /// # Returns
    /// `true` if the toolbar position was updated, `false` if not dragging
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

    /// Constrains the toolbar position within the specified bounds.
    ///
    /// This method ensures the toolbar stays within the visible area by clamping
    /// its position to stay at least `TOOLBAR_EDGE_PADDING` pixels from the edges.
    /// Used after drag operations to prevent the toolbar from being dragged off-screen.
    ///
    /// # Arguments
    /// * `max_x` - Maximum X-coordinate (typically screen width)
    /// * `max_y` - Maximum Y-coordinate (typically screen height)
    pub fn clamp_within(&mut self, max_x: f64, max_y: f64) {
        self.x = self
            .x
            .clamp(TOOLBAR_EDGE_PADDING, max_x.max(TOOLBAR_EDGE_PADDING));
        self.y = self
            .y
            .clamp(TOOLBAR_EDGE_PADDING, max_y.max(TOOLBAR_EDGE_PADDING));
    }

    /// Terminates the current toolbar drag operation.
    ///
    /// This method resets the dragging state and clears the mouse position tracking,
    /// allowing new drag operations to start cleanly.
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
