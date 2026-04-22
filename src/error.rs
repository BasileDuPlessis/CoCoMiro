//! # Error Types Module
//!
//! This module defines the custom error types used throughout the CoCoMiro application.
//! It provides specific error variants for different failure scenarios and proper error
//! handling infrastructure for both host and WebAssembly targets.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Custom error types for the CoCoMiro application.
///
/// This enum provides specific error types for different failure scenarios,
/// allowing for better error handling and recovery strategies.
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    /// Browser environment errors (missing window, document, etc.)
    BrowserEnv(String),
    /// Canvas-related errors (context creation, rendering failures)
    Canvas(String),
    /// DOM manipulation errors (element access, property setting)
    Dom(String),
    /// Event handling errors (listener attachment failures)
    Event(String),
    /// State management errors (invalid state transitions)
    State(String),
    /// Rendering errors (drawing failures, context issues)
    Render(String),
    /// Generic application errors
    Generic(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BrowserEnv(msg) => write!(f, "Browser environment error: {}", msg),
            AppError::Canvas(msg) => write!(f, "Canvas error: {}", msg),
            AppError::Dom(msg) => write!(f, "DOM error: {}", msg),
            AppError::Event(msg) => write!(f, "Event error: {}", msg),
            AppError::State(msg) => write!(f, "State error: {}", msg),
            AppError::Render(msg) => write!(f, "Render error: {}", msg),
            AppError::Generic(msg) => write!(f, "Application error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[cfg(target_arch = "wasm32")]
impl From<JsValue> for AppError {
    fn from(js_error: JsValue) -> Self {
        AppError::Generic(format!("JavaScript error: {:?}", js_error))
    }
}

#[cfg(target_arch = "wasm32")]
impl From<AppError> for JsValue {
    fn from(error: AppError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

/// Result type alias for operations that may fail.
pub type AppResult<T> = Result<T, AppError>;
