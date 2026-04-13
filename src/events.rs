//! # Event Handling System
//!
//! This module provides a unified interface to the event handling system.
//! It re-exports functions from specialized event handling modules for
//! backward compatibility and convenience.
//!
//! ## Module Organization
//!
//! The event system is now split into focused modules:
//! - **event_constants**: Shared constants used across event handlers
//! - **mouse_events**: Mouse interaction handlers (click, drag, wheel, etc.)
//! - **keyboard_events**: Keyboard shortcut handlers
//! - **text_input**: Text editing overlay functionality
//! - **event_setup**: Main event listener setup and shared utilities

// Re-export main setup function for backward compatibility
#[cfg(target_arch = "wasm32")]
pub use crate::event_setup::setup_event_listeners;

// Re-export utility functions for backward compatibility
#[cfg(target_arch = "wasm32")]
pub use crate::event_setup::{
    end_drag_if_needed, end_toolbar_drag_if_needed, js_error_to_app_error,
};

// Re-export text input functionality
#[cfg(target_arch = "wasm32")]
pub use crate::text_input::create_text_input_overlay;
