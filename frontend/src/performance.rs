/// Performance logger for tracking rendering and canvas operations
pub struct PerformanceLogger;

impl PerformanceLogger {
    /// Log render time for a component
    pub fn log_render_time(component: &str, start: f64) {
        let duration = Self::get_current_time() - start;
        log::info!("{} render time: {:.2}ms", component, duration);
    }

    /// Log canvas operation performance
    pub fn log_canvas_operation(operation: &str, start: f64) {
        let duration = Self::get_current_time() - start;
        if duration > 16.0 {
            // Slower than 60fps
            log::warn!("Slow canvas operation '{}': {:.2}ms", operation, duration);
        } else {
            log::debug!("Canvas operation '{}': {:.2}ms", operation, duration);
        }
    }

    /// Log state changes
    pub fn log_state_change(component: &str, action: &str) {
        log::debug!("State change in {}: {}", component, action);
    }

    /// Log user interactions
    pub fn log_user_interaction(interaction: &str, details: &str) {
        log::info!("User interaction: {} - {}", interaction, details);
    }

    /// Log errors with context
    pub fn log_error(context: &str, error: &str) {
        log::error!("Error in {}: {}", context, error);
    }

    /// Get current time in milliseconds
    fn get_current_time() -> f64 {
        js_sys::Date::now()
    }
}

/// Macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $context:expr) => {{
        let start = js_sys::Date::now();
        let result = $operation;
        let duration = js_sys::Date::now() - start;
        log::debug!("Operation '{}' took {:.2}ms", $context, duration);
        result
    }};
}
