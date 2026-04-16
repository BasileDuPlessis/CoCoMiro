use super::types::{
    DEFAULT_NOTE_HEIGHT, DEFAULT_NOTE_WIDTH, NEXT_ID, Ordering, RESIZE_HANDLE_SIZE, ResizeHandle,
    TextFormat,
};

#[derive(Debug, Clone, PartialEq)]
/// Represents a single sticky note on the infinite canvas.
///
/// Each sticky note has a unique ID, position in world coordinates,
/// dimensions, text content, formatting information, and background color.
/// Notes are rendered as rectangles with text content and can be dragged around the canvas.
pub struct StickyNote {
    /// Unique identifier for this note (auto-generated)
    pub id: u32,
    /// X-coordinate of the top-left corner in world space
    pub x: f64,
    /// Y-coordinate of the top-left corner in world space
    pub y: f64,
    /// Width of the note in world units
    pub width: f64,
    /// Height of the note in world units
    pub height: f64,
    /// Text content displayed on the note
    pub content: String,
    /// Rich text formatting spans applied to the content
    pub formatting: Vec<TextFormat>,
    /// Background color as a hex string (e.g., "#ffff88")
    pub color: String,
}

impl StickyNote {
    /// Creates a new sticky note at the specified world coordinates.
    ///
    /// The note is assigned a unique ID, default dimensions (200x150),
    /// default content ("New note"), no formatting, and default color (#ffff88).
    ///
    /// # Arguments
    /// * `x` - X-coordinate of the top-left corner in world space
    /// * `y` - Y-coordinate of the top-left corner in world space
    ///
    /// # Returns
    /// A new `StickyNote` instance with default properties
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            x,
            y,
            width: DEFAULT_NOTE_WIDTH,
            height: DEFAULT_NOTE_HEIGHT,
            content: "New note".to_string(),
            formatting: Vec::new(),
            color: "#ffff88".to_string(),
        }
    }

    /// Checks if the given point lies within this note's boundaries.
    ///
    /// This method is used for hit testing during mouse interactions,
    /// determining whether a click or drag operation should affect this note.
    ///
    /// # Arguments
    /// * `px` - X-coordinate of the test point in world space
    /// * `py` - Y-coordinate of the test point in world space
    ///
    /// # Returns
    /// `true` if the point is inside the note's rectangle, `false` otherwise
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    /// Returns the world-space position of a resize handle on this note.
    ///
    /// The handle position is calculated relative to the note's top-left corner
    /// and dimensions. The returned coordinates represent the center of the handle.
    ///
    /// # Arguments
    /// * `handle` - The resize handle to get the position for
    ///
    /// # Returns
    /// A tuple `(x, y)` representing the world coordinates of the handle center
    pub fn handle_position(&self, handle: ResizeHandle) -> (f64, f64) {
        match handle {
            ResizeHandle::TopLeft => (self.x, self.y),
            ResizeHandle::Top => (self.x + self.width / 2.0, self.y),
            ResizeHandle::TopRight => (self.x + self.width, self.y),
            ResizeHandle::Right => (self.x + self.width, self.y + self.height / 2.0),
            ResizeHandle::BottomRight => (self.x + self.width, self.y + self.height),
            ResizeHandle::Bottom => (self.x + self.width / 2.0, self.y + self.height),
            ResizeHandle::BottomLeft => (self.x, self.y + self.height),
            ResizeHandle::Left => (self.x, self.y + self.height / 2.0),
        }
    }

    /// Returns the screen-space bounding box of a resize handle.
    ///
    /// This method converts the handle's world position to screen coordinates
    /// and returns the bounding box for hit testing. The handle is treated as
    /// a square centered on the handle position.
    ///
    /// # Arguments
    /// * `handle` - The resize handle to get the bounds for
    /// * `viewport` - The current viewport state for coordinate transformation
    /// * `canvas_width` - The canvas width in pixels
    /// * `canvas_height` - The canvas height in pixels
    ///
    /// # Returns
    /// A tuple `(left, top, right, bottom)` representing the screen bounding box
    pub fn handle_bounds(
        &self,
        handle: ResizeHandle,
        viewport: &crate::viewport::ViewportState,
        canvas_width: f64,
        canvas_height: f64,
    ) -> (f64, f64, f64, f64) {
        let (world_x, world_y) = self.handle_position(handle);
        let screen_x = world_x * viewport.zoom + canvas_width / 2.0 + viewport.pan_x;
        let screen_y = world_y * viewport.zoom + canvas_height / 2.0 + viewport.pan_y;

        let half_size = RESIZE_HANDLE_SIZE / 2.0;
        (
            screen_x - half_size,
            screen_y - half_size,
            screen_x + half_size,
            screen_y + half_size,
        )
    }

    /// Returns the screen-space positions of all resize handles for this note.
    ///
    /// This method converts all handle world positions to screen coordinates
    /// for rendering and hit testing purposes.
    ///
    /// # Arguments
    /// * `viewport` - The current viewport state for coordinate transformation
    /// * `canvas_width` - The canvas width in pixels
    /// * `canvas_height` - The canvas height in pixels
    ///
    /// # Returns
    /// A vector of tuples `(handle, x, y)` where `handle` is the ResizeHandle variant
    /// and `(x, y)` are the screen coordinates of the handle center
    pub fn handle_positions(
        &self,
        viewport: &crate::viewport::ViewportState,
        canvas_width: f64,
        canvas_height: f64,
    ) -> Vec<(ResizeHandle, f64, f64)> {
        [
            ResizeHandle::TopLeft,
            ResizeHandle::Top,
            ResizeHandle::TopRight,
            ResizeHandle::Right,
            ResizeHandle::BottomRight,
            ResizeHandle::Bottom,
            ResizeHandle::BottomLeft,
            ResizeHandle::Left,
        ]
        .iter()
        .map(|&handle| {
            let (world_x, world_y) = self.handle_position(handle);
            let screen_x = world_x * viewport.zoom + canvas_width / 2.0 + viewport.pan_x;
            let screen_y = world_y * viewport.zoom + canvas_height / 2.0 + viewport.pan_y;
            (handle, screen_x, screen_y)
        })
        .collect()
    }
}
