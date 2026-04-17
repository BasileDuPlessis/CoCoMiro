use std::sync::atomic::AtomicU32;

/// Default width for new sticky notes (in world units)
pub const DEFAULT_NOTE_WIDTH: f64 = 200.0;
/// Default height for new sticky notes (in world units)
pub const DEFAULT_NOTE_HEIGHT: f64 = 150.0;
/// Size of resize handles (in screen pixels)
pub const RESIZE_HANDLE_SIZE: f64 = 8.0;

#[derive(Debug, Clone, PartialEq)]
/// Represents formatting information for a range of text in a sticky note.
///
/// Each format span defines the start and end character positions in the content
/// string and the formatting flags to apply (bold, italic, underline).
pub struct TextFormat {
    /// Start character position (inclusive) in the content string
    pub start: usize,
    /// End character position (exclusive) in the content string
    pub end: usize,
    /// Whether this range should be rendered in bold
    pub bold: bool,
    /// Whether this range should be rendered in italic
    pub italic: bool,
    /// Whether this range should be rendered with underline
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the different resize handles available on a sticky note.
///
/// Each variant corresponds to a specific position on the note's border where
/// users can click and drag to resize the note. The handles allow resizing
/// from corners (diagonal resize) and edges (horizontal/vertical resize).
pub enum ResizeHandle {
    /// Top-left corner handle for diagonal resizing
    TopLeft,
    /// Top edge handle for vertical resizing
    Top,
    /// Top-right corner handle for diagonal resizing
    TopRight,
    /// Right edge handle for horizontal resizing
    Right,
    /// Bottom-right corner handle for diagonal resizing
    BottomRight,
    /// Bottom edge handle for vertical resizing
    Bottom,
    /// Bottom-left corner handle for diagonal resizing
    BottomLeft,
    /// Left edge handle for horizontal resizing
    Left,
}

impl ResizeHandle {
    /// Returns the CSS cursor style appropriate for this resize handle.
    ///
    /// The cursor indicates the type of resize operation that will occur
    /// when dragging this handle (diagonal vs horizontal/vertical).
    pub fn cursor(&self) -> &'static str {
        match self {
            ResizeHandle::TopLeft | ResizeHandle::BottomRight => "nw-resize",
            ResizeHandle::Top | ResizeHandle::Bottom => "n-resize",
            ResizeHandle::TopRight | ResizeHandle::BottomLeft => "ne-resize",
            ResizeHandle::Left | ResizeHandle::Right => "e-resize",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// Tracks the state of an ongoing resize operation for sticky notes.
///
/// This struct maintains information about which note is being resized,
/// which handle is being used, the original dimensions, and the starting
/// mouse position for the resize operation.
pub struct ResizingState {
    /// Whether a resize operation is currently active
    pub is_resizing: bool,
    /// ID of the note being resized (None if not resizing)
    pub note_id: Option<u32>,
    /// The handle being used for resizing (None if not resizing)
    pub handle: Option<ResizeHandle>,
    /// Mouse X position when resize started (screen coordinates)
    pub start_mouse_x: f64,
    /// Mouse Y position when resize started (screen coordinates)
    pub start_mouse_y: f64,
    /// Original width of the note before resizing started
    pub original_width: f64,
    /// Original height of the note before resizing started
    pub original_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Parameters for a resize operation on a sticky note.
///
/// This struct bundles all the necessary data for performing a resize operation,
/// including the handle being used, mouse positions, and original dimensions.
pub struct ResizeParams {
    /// The resize handle being used
    pub handle: ResizeHandle,
    /// Mouse X position when resize started (screen coordinates)
    pub start_mouse_x: f64,
    /// Mouse Y position when resize started (screen coordinates)
    pub start_mouse_y: f64,
    /// Current mouse X position (screen coordinates)
    pub current_mouse_x: f64,
    /// Current mouse Y position (screen coordinates)
    pub current_mouse_y: f64,
    /// Original width of the note before resizing
    pub original_width: f64,
    /// Original height of the note before resizing
    pub original_height: f64,
}

pub static NEXT_ID: AtomicU32 = AtomicU32::new(1);
pub use std::sync::atomic::Ordering;
