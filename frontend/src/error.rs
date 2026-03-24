use std::fmt;
use wasm_bindgen::JsValue;

/// Custom error type for canvas and DOM operations
#[derive(Debug, Clone)]
pub enum CanvasError {
    CanvasNotFound,
    ContextNotAvailable,
    ElementCastFailed(String),
    WindowAccessFailed,
    SyntheticEventFailed(String),
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasError::CanvasNotFound => write!(f, "Canvas element not found"),
            CanvasError::ContextNotAvailable => write!(f, "Canvas 2D context not available"),
            CanvasError::ElementCastFailed(type_name) => {
                write!(f, "Failed to cast element to {}", type_name)
            }
            CanvasError::WindowAccessFailed => write!(f, "Failed to access window object"),
            CanvasError::SyntheticEventFailed(event_type) => {
                write!(f, "Failed to create synthetic {} event", event_type)
            }
        }
    }
}

impl std::error::Error for CanvasError {}

/// Result type alias for canvas operations
pub type CanvasResult<T> = Result<T, CanvasError>;

/// Convert JsValue errors to CanvasError
impl From<JsValue> for CanvasError {
    fn from(_: JsValue) -> Self {
        CanvasError::ContextNotAvailable
    }
}

/// Log an error and return a default value
pub fn log_error_and_default<T: Default>(error: CanvasError, context: &str) -> T {
    log::error!("{}: {}", context, error);
    T::default()
}

/// Log an error and return None
pub fn log_error_and_none<T>(error: CanvasError, context: &str) -> Option<T> {
    log::error!("{}: {}", context, error);
    None
}
