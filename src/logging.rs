//! # Logging Module
//!
//! This module provides logging functionality for the CoCoMiro application.
//! All logging functions are conditionally compiled for WebAssembly targets
//! to ensure compatibility with browser environments.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys;

#[cfg(target_arch = "wasm32")]
/// Logs JavaScript errors to the browser console with enhanced context.
///
/// This function provides consistent error logging for WebAssembly code,
/// ensuring errors are visible in browser developer tools with additional context.
///
/// # Arguments
/// * `context` - Descriptive context for where the error occurred
/// * `error` - The error to log (can be AppError or JsValue)
pub fn log_js_error(context: &str, error: &impl std::fmt::Display) {
    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro [{context}]: {error}"
    )));
}

#[cfg(target_arch = "wasm32")]
/// Logs raw JavaScript values as errors.
///
/// This function is specifically for logging JsValue errors that don't
/// implement Display but can be debug-formatted.
///
/// # Arguments
/// * `context` - Descriptive context for where the error occurred
/// * `error` - The JsValue error to log
pub fn log_jsvalue_error(context: &str, error: &JsValue) {
    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro [{context}]: {:?}",
        error
    )));
}

#[cfg(target_arch = "wasm32")]
/// Logs application errors with recovery suggestions.
///
/// This function logs errors and provides user-friendly recovery information
/// when possible. For critical errors, it may suggest page refresh.
///
/// # Arguments
/// * `error` - The application error to log
/// * `operation` - Description of the operation that failed
pub fn log_app_error(error: &crate::error::AppError, operation: &str) {
    let recovery_hint = match error {
        crate::error::AppError::BrowserEnv(_) => {
            "Try refreshing the page or checking your browser compatibility."
        }
        crate::error::AppError::Canvas(_) => "Try resizing the window or refreshing the page.",
        crate::error::AppError::Dom(_) => "The page may have been modified. Try refreshing.",
        crate::error::AppError::Event(_) => "Interaction may be limited. Try refreshing the page.",
        crate::error::AppError::State(_) => {
            "Application state may be corrupted. Try refreshing the page."
        }
        crate::error::AppError::Render(_) => "Rendering failed. Try refreshing the page.",
        crate::error::AppError::Generic(_) => {
            "An unexpected error occurred. Try refreshing the page."
        }
    };

    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro Error during '{}': {}\nRecovery: {}",
        operation, error, recovery_hint
    )));
}

#[cfg(target_arch = "wasm32")]
/// Logs informational messages to the browser console.
///
/// # Arguments
/// * `message` - The message to log
pub fn log_info(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
/// Logs warning messages to the browser console.
///
/// # Arguments
/// * `message` - The warning message to log
pub fn log_warn(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}
