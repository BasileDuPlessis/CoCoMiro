//! # Authentication Module
//!
//! This module handles user authentication state management for the CoCoMiro application.
//! It provides structures and functions for tracking authentication status and user information.

/// Represents an authenticated user
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// Unique user identifier from Google
    pub id: String,
    /// User's email address
    pub email: String,
    /// User's display name
    pub name: String,
    /// URL to user's profile picture
    pub picture: Option<String>,
}

impl User {
    /// Create a new User instance
    pub fn new(id: String, email: String, name: String, picture: Option<String>) -> Self {
        Self {
            id,
            email,
            name,
            picture,
        }
    }
}

/// Authentication state of the application
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    /// User is not authenticated
    Unauthenticated,
    /// User is authenticated with the provided user information
    Authenticated(User),
    /// Authentication is in progress (e.g., OAuth flow)
    Authenticating,
    /// Authentication failed with an error message
    Error(String),
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState::Unauthenticated
    }
}

impl AuthState {
    /// Check if the user is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }

    /// Check if authentication is in progress
    pub fn is_authenticating(&self) -> bool {
        matches!(self, AuthState::Authenticating)
    }

    /// Check if there's an authentication error
    pub fn has_error(&self) -> bool {
        matches!(self, AuthState::Error(_))
    }

    /// Get the authenticated user, if any
    pub fn user(&self) -> Option<&User> {
        match self {
            AuthState::Authenticated(user) => Some(user),
            _ => None,
        }
    }

    /// Get the error message, if any
    pub fn error_message(&self) -> Option<&str> {
        match self {
            AuthState::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Authentication manager for handling auth state transitions
#[derive(Debug, Clone)]
pub struct AuthManager {
    state: AuthState,
}

impl AuthManager {
    /// Create a new AuthManager with default unauthenticated state
    pub fn new() -> Self {
        Self {
            state: AuthState::default(),
        }
    }

    /// Get the current authentication state
    pub fn state(&self) -> &AuthState {
        &self.state
    }

    /// Set the authentication state to authenticated with a user
    pub fn set_authenticated(&mut self, user: User) {
        self.state = AuthState::Authenticated(user);
    }

    /// Set the authentication state to unauthenticated
    pub fn set_unauthenticated(&mut self) {
        self.state = AuthState::Unauthenticated;
    }

    /// Set the authentication state to authenticating
    pub fn set_authenticating(&mut self) {
        self.state = AuthState::Authenticating;
    }

    /// Set the authentication state to error with a message
    pub fn set_error(&mut self, message: String) {
        self.state = AuthState::Error(message);
    }

    /// Clear any error state, resetting to unauthenticated
    pub fn clear_error(&mut self) {
        if self.state.has_error() {
            self.state = AuthState::Unauthenticated;
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use js_sys::{Reflect, Array, Object, Function};

#[cfg(target_arch = "wasm32")]
impl AuthManager {
    /// Initialize authentication by checking stored tokens
    /// This should be called on app startup
    pub fn initialize_from_storage(&mut self) -> Result<(), String> {
        // Call JavaScript to check stored authentication
        match self.call_js_function("GoogleAuth.restoreState", &[]) {
            Ok(_) => {
                // JavaScript will call back via notifyAuthRestored if authenticated
                Ok(())
            }
            Err(err) => {
                log::warn!("Failed to restore auth state from storage: {:?}", err);
                self.set_unauthenticated();
                Ok(())
            }
        }
    }

    /// Trigger the Google OAuth login flow
    pub fn login(&mut self) -> Result<(), JsValue> {
        self.set_authenticating();

        // Call JavaScript to trigger Google OAuth
        self.call_js_function("GoogleAuth.login", &[])
    }

    /// Logout the current user
    pub fn logout(&mut self) -> Result<(), JsValue> {
        // Call JavaScript to clear stored data
        match self.call_js_function("GoogleAuth.logout", &[]) {
            Ok(_) => {
                self.set_unauthenticated();
                Ok(())
            }
            Err(err) => {
                log::error!("Failed to logout: {:?}", err);
                // Still set unauthenticated even if JS call failed
                self.set_unauthenticated();
                Err(err)
            }
        }
    }

    /// Handle successful authentication from JavaScript callback
    pub fn handle_auth_success(&mut self, user: User) {
        log::info!("Authentication successful for user: {}", user.name);
        self.set_authenticated(user);
    }

    /// Handle authentication error from JavaScript callback
    pub fn handle_auth_error(&mut self, error: String) {
        log::error!("Authentication error: {}", error);
        self.set_error(error);
    }

    /// Check if user is authenticated by calling JavaScript
    pub fn check_auth_status(&mut self) -> Result<bool, JsValue> {
        match self.call_js_function("GoogleAuth.isAuthenticated", &[]) {
            Ok(result) => {
                // Parse the boolean result from JavaScript
                result.as_bool().ok_or_else(|| {
                    JsValue::from_str("Expected boolean result from isAuthenticated")
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Get current user info from JavaScript
    pub fn get_current_user(&self) -> Result<Option<User>, JsValue> {
        match self.call_js_function("GoogleAuth.getUser", &[]) {
            Ok(result) => {
                if result.is_null() || result.is_undefined() {
                    Ok(None)
                } else {
                    // Parse user object from JavaScript
                    self.parse_user_from_js(result)
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Helper method to call JavaScript functions
    fn call_js_function(&self, function_path: &str, args: &[&JsValue]) -> Result<JsValue, JsValue> {
        // Access the global window object
        let window = web_sys::window().ok_or("No global window object")?;

        // Navigate to the function (e.g., "GoogleAuth.login")
        let mut obj = window.into();
        let path_parts: Vec<&str> = function_path.split('.').collect();

        // Navigate through the object path
        for part in &path_parts[..path_parts.len() - 1] {
            obj = Reflect::get(&obj, &JsValue::from_str(part))?
                .dyn_into::<Object>()?;
        }

        // Get the function
        let function_name = path_parts.last().unwrap();
        let function = Reflect::get(&obj, &JsValue::from_str(function_name))?
            .dyn_into::<Function>()?;

        // Call the function with arguments
        let js_args = Array::from_iter(args.iter().cloned());
        function.apply(&obj, &js_args)
    }

    /// Parse user object from JavaScript result
    fn parse_user_from_js(&self, js_value: JsValue) -> Result<Option<User>, JsValue> {
        if js_value.is_null() || js_value.is_undefined() {
            return Ok(None);
        }

        let obj = js_value.dyn_into::<Object>()?;

        let id = Reflect::get(&obj, &JsValue::from_str("id"))?
            .as_string()
            .ok_or("Missing or invalid id field")?;

        let email = Reflect::get(&obj, &JsValue::from_str("email"))?
            .as_string()
            .ok_or("Missing or invalid email field")?;

        let name = Reflect::get(&obj, &JsValue::from_str("name"))?
            .as_string()
            .ok_or("Missing or invalid name field")?;

        let picture = Reflect::get(&obj, &JsValue::from_str("picture"))?
            .as_string(); // Optional field

        Ok(Some(User::new(id, email, name, picture)))
    }
}
