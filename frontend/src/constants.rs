// Constants for the Mori infinite canvas application

// Toolbar
pub const TOOLBAR_INITIAL_X: f64 = 10.0;
pub const TOOLBAR_INITIAL_Y: f64 = 10.0;
pub const TOOLBAR_GAP: &str = "5px";
pub const TOOLBAR_PADDING: &str = "8px";
pub const TOOLBAR_HANDLE_HEIGHT: &str = "8px";
pub const TOOLBAR_HANDLE_MARGIN: &str = "-8px -8px 8px -8px";

// Zoom and pan
pub const ZOOM_FACTOR_IN: f64 = 1.2;
pub const ZOOM_FACTOR_OUT: f64 = 0.8333; // 1.0 / 1.2
pub const MAX_ZOOM: f64 = 10.0;
pub const MIN_ZOOM: f64 = 0.1;

// Grid
pub const GRID_BASE_SPACING: f64 = 50.0;
pub const GRID_LINE_WIDTH_MIN: f64 = 0.5;
pub const GRID_LINE_WIDTH_MAX: f64 = 2.0;

// Canvas
pub const CANVAS_MAX_WIDTH: u32 = 3000;
pub const CANVAS_MAX_HEIGHT: u32 = 2000;

// Sticky notes
pub const STICKY_NOTE_DEFAULT_WIDTH: f64 = 200.0;
pub const STICKY_NOTE_DEFAULT_HEIGHT: f64 = 150.0;
pub const STICKY_NOTE_CENTER_OFFSET_X: f64 = 100.0; // width / 2
pub const STICKY_NOTE_CENTER_OFFSET_Y: f64 = 75.0; // height / 2

// Styling
pub const FONT_SIZE_BASE: f64 = 16.0;
pub const LINE_MARGIN: f64 = 4.0;
pub const DEBUG_OVERLAY_BG: &str = "rgba(0, 0, 0, 0.7)";
pub const DEBUG_OVERLAY_TEXT: &str = "#FFFFFF";
pub const GRID_COLOR: &str = "#E0E0E0";
pub const STICKY_NOTE_BG: &str = "#FFFF88";
pub const STICKY_NOTE_BORDER: &str = "#CCCC00";
pub const STICKY_NOTE_DRAG_SHADOW: &str = "0 4px 8px rgba(0, 0, 0, 0.2)";
pub const STICKY_NOTE_DRAG_OPACITY: &str = "0.8";

// Button styles
pub const BUTTON_SIZE: &str = "32px";
pub const BUTTON_BORDER: &str = "1px solid #ccc";
pub const BUTTON_BG: &str = "white";
pub const STICKY_BUTTON_BG: &str = "#FFFF88";

// Canvas
pub const CANVAS_BG: &str = "#FFFFFF";
