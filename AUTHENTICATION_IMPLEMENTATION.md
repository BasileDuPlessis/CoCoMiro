# Google Authentication Implementation for CoCoMiro

## Overview

This document outlines the complete implementation plan for adding Google OAuth authentication to the CoCoMiro infinite canvas application. The implementation follows an incremental approach to ensure the app remains functional at each step.

## Requirements

### Functional Requirements
- Users must authenticate with Google OAuth before accessing the canvas
- Unauthenticated users see a login overlay and cannot interact with the canvas
- Authentication state persists across browser sessions
- Clean logout functionality with proper session cleanup
- Error handling for OAuth failures and network issues

### Technical Requirements
- Google OAuth Client ID: `782746094433-c9bcjvl9stv1jnqmkklu2u07bpjn4vvo.apps.googleusercontent.com`
- Token storage in both localStorage (persistent) and sessionStorage (current session)
- Automatic token refresh when needed
- Secure JWT token validation
- WebAssembly-compatible implementation

## Implementation Steps

### Step 1: Rust-side Authentication State ✅ COMPLETED

**Objective**: Implement core authentication state management in Rust.

**Files Modified**:
- `src/auth.rs` (new file)
- `src/lib.rs` (modified)

**Implementation Details**:

1. **Created `src/auth.rs`** with:
   - `User` struct for authenticated user data
   - `AuthState` enum with states: Unauthenticated, Authenticated, Authenticating, Error
   - `AuthManager` struct for state transitions

2. **Modified `src/lib.rs`**:
   - Added `auth` module import
   - Added `auth: AuthManager` field to `AppState`
   - Updated `AppState::default()` to initialize auth manager

**Key Features**:
- Type-safe authentication state management
- Clean API for auth state transitions
- WebAssembly-compatible with conditional compilation
- Placeholder methods for JavaScript interop

**Testing**:
- ✅ Code compiles without errors
- ✅ All existing tests pass (50 unit + 14 integration)
- ✅ No breaking changes to existing functionality

### Step 2: JavaScript Google Identity Services Integration

**Objective**: Create fresh JavaScript integration with Google Identity Services.

**Files to Create/Modify**:
- `src/auth.js` (new source file)
- `index.html` (add Google Identity Services script)
- `src/auth.rs` (implement JS interop methods)

**Implementation Details**:

1. **Create `src/auth.js`**:
   - Initialize Google Identity Services with client ID
   - Handle OAuth callback and token processing
   - Implement login/logout functions
   - Token storage in localStorage/sessionStorage
   - JWT validation and expiration checking

2. **Modify `index.html`**:
   - Add Google Identity Services script tag
   - Ensure proper loading order with WASM

3. **Enhance `src/auth.rs`**:
   - Implement `initialize_from_storage()` method
   - Implement `login()` and `logout()` with JS calls
   - Add `handle_auth_success()` and `handle_auth_error()` callbacks

**Key Features**:
- Modern Google Identity Services API (not deprecated Google Sign-In)
- Proper OAuth 2.0 flow with PKCE
- Secure token storage and validation
- Error handling for network issues

### Step 3: Authentication UI Elements (Login Overlay)

**Objective**: Add login overlay UI that blocks canvas access for unauthenticated users.

**Files to Create/Modify**:
- `src/app.rs` (modify app markup)
- `styles.css` (add login overlay styles)
- `src/auth.rs` (UI state integration)

**Implementation Details**:

1. **Modify `src/app.rs`**:
   - Add login overlay HTML to `app_markup()`
   - Include Google sign-in button
   - Add status messages and error display

2. **Update `styles.css`**:
   - Style login overlay (full-screen, centered)
   - Google branding compliance
   - Responsive design
   - Loading states and animations

3. **Integration**:
   - Show/hide overlay based on auth state
   - Block canvas interaction when unauthenticated
   - Update status messages dynamically

**Key Features**:
- Professional login overlay design
- Google OAuth button styling
- Loading indicators during authentication
- Error message display

### Step 4: Authentication Flow Integration

**Objective**: Integrate authentication checks throughout the app lifecycle.

**Files to Modify**:
- `src/lib.rs` (startup authentication initialization)
- `src/events.rs` (block interactions when unauthenticated)
- `src/canvas.rs` (conditional rendering)
- `src/toolbar.rs` (add login/logout buttons)

**Implementation Details**:

1. **App Startup**:
   - Initialize auth state from stored tokens
   - Show login overlay if unauthenticated
   - Restore previous session if valid

2. **Event Handling**:
   - Block canvas interactions when unauthenticated
   - Allow only login-related interactions
   - Handle auth state changes

3. **UI Updates**:
   - Add login/logout button to toolbar
   - Update status bar with auth information
   - Show user info when authenticated

**Key Features**:
- Seamless authentication flow
- Proper session restoration
- Non-intrusive auth state management

### Step 5: Token Management & Error Handling

**Objective**: Implement robust token management and comprehensive error handling.

**Files to Modify**:
- `src/auth.js` (token refresh logic)
- `src/auth.rs` (error state management)
- `src/lib.rs` (error recovery)

**Implementation Details**:

1. **Token Management**:
   - Automatic token refresh before expiration
   - Dual storage (localStorage + sessionStorage)
   - Secure token validation
   - Cleanup on logout

2. **Error Handling**:
   - OAuth flow failures
   - Network connectivity issues
   - Token expiration handling
   - Graceful degradation

3. **Security**:
   - JWT signature validation
   - Secure token storage
   - XSS protection
   - Proper error messages (no sensitive data leakage)

**Key Features**:
- Robust session management
- Comprehensive error recovery
- Security best practices
- User-friendly error messages

## Testing Strategy

### Unit Tests
- Authentication state transitions
- Token validation logic
- Error handling scenarios

### Integration Tests
- Full authentication flow
- Session persistence
- Error recovery

### End-to-End Tests
- Complete user journey
- Browser compatibility
- Network failure scenarios

### Visual Regression Tests
- Login overlay appearance
- Authentication state UI changes

## Security Considerations

1. **Token Storage**: Use httpOnly cookies for production, localStorage only for development
2. **CORS**: Proper CORS configuration for OAuth callbacks
3. **XSS Protection**: Sanitize all user data and tokens
4. **CSRF**: Implement CSRF protection for auth endpoints
5. **Audit Logging**: Log authentication events for security monitoring

## Deployment Checklist

- [ ] Google OAuth client ID configured for production domain
- [ ] HTTPS enabled (required for OAuth)
- [ ] CORS properly configured
- [ ] Token storage security reviewed
- [ ] Error handling tested in production-like environment
- [ ] User session management verified
- [ ] Logout functionality tested across browsers

## Rollback Plan

If authentication implementation needs to be disabled:

1. Remove auth module imports
2. Remove auth field from AppState
3. Remove login overlay from HTML
4. Remove Google Identity Services script
5. Restore original toolbar and event handling

## Future Enhancements

- Multi-factor authentication
- Social login providers (GitHub, etc.)
- User profile management
- Session timeout configuration
- Admin user management
- Audit logging and compliance features

## Success Criteria

- [ ] Users can authenticate with Google OAuth
- [ ] Authentication state persists across sessions
- [ ] Unauthenticated users cannot access canvas
- [ ] Clean logout functionality
- [ ] Proper error handling for all failure scenarios
- [ ] All existing functionality preserved
- [ ] Code follows project best practices
- [ ] Comprehensive test coverage
- [ ] Security audit passed