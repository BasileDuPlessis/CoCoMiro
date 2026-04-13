//! # Event System Constants
//!
//! This module contains constants used throughout the event handling system.

#[cfg(target_arch = "wasm32")]
/// Zoom factor applied per wheel event (1.1 = 10% zoom per step)
pub const ZOOM_STEP_FACTOR: f64 = 1.1;

#[cfg(target_arch = "wasm32")]
/// Distance moved per keyboard pan event (in screen pixels)
pub const KEYBOARD_PAN_STEP: f64 = 40.0;