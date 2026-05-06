/**
 * Google Identity Services integration for CoCoMiro
 *
 * This module handles the OAuth flow using Google's Identity Services JavaScript API.
 * It provides functions to initialize the OAuth client, trigger login, and manage tokens.
 */

// Global variables for GIS
let googleAuth = null;
let currentUser = null;

// Configuration - Google OAuth client ID
const GOOGLE_CLIENT_ID = '782746094433-c9bcjvl9stv1jnqmkklu2u07bpjn4vvo.apps.googleusercontent.com';

/**
 * Initialize Google Identity Services
 * Call this after the GIS script has loaded
 */
function initializeGoogleAuth(clientId) {
    if (!window.google) {
        console.error('Google Identity Services script not loaded');
        return false;
    }

    try {
        googleAuth = google.accounts.id.initialize({
            client_id: clientId || GOOGLE_CLIENT_ID,
            callback: handleCredentialResponse,
            auto_select: false,
            cancel_on_tap_outside: true,
            context: 'signin',
            ux_mode: 'popup',
            use_fedcm_for_prompt: true,
        });

        console.log('Google Identity Services initialized successfully');
        return true;
    } catch (error) {
        console.error('Failed to initialize Google Identity Services:', error);
        return false;
    }
}

/**
 * Handle the credential response from Google OAuth
 */
function handleCredentialResponse(response) {
    try {
        console.log('Received credential response from Google');

        // Decode the JWT token to get user information
        const payload = parseJwt(response.credential);

        const userInfo = {
            id: payload.sub,
            email: payload.email,
            name: payload.name,
            picture: payload.picture,
            email_verified: payload.email_verified,
            given_name: payload.given_name,
            family_name: payload.family_name,
        };

        // Store tokens securely
        storeAuthTokens(response.credential, userInfo);

        // Update current user
        currentUser = userInfo;

        // Notify the Rust application
        if (window.notifyAuthSuccess) {
            window.notifyAuthSuccess(userInfo);
        }

        console.log('Authentication successful for user:', userInfo.name);
    } catch (error) {
        console.error('Failed to handle credential response:', error);
        if (window.notifyAuthError) {
            window.notifyAuthError('Failed to process authentication response: ' + error.message);
        }
    }
}

/**
 * Store authentication tokens and user info
 */
function storeAuthTokens(token, userInfo) {
    try {
        // Store the full JWT token
        localStorage.setItem('cocomiro_google_id_token', token);
        sessionStorage.setItem('cocomiro_google_id_token', token);

        // Store user info
        const userInfoStr = JSON.stringify(userInfo);
        localStorage.setItem('cocomiro_user_info', userInfoStr);
        sessionStorage.setItem('cocomiro_user_info', userInfoStr);

        // Store timestamp for token validation
        const timestamp = Date.now();
        localStorage.setItem('cocomiro_auth_timestamp', timestamp.toString());
        sessionStorage.setItem('cocomiro_auth_timestamp', timestamp.toString());

        console.log('Authentication tokens stored securely');
    } catch (error) {
        console.error('Failed to store auth tokens:', error);
        throw error;
    }
}

/**
 * Trigger the Google OAuth login flow
 */
function triggerGoogleLogin() {
    if (!googleAuth) {
        const error = 'Google Auth not initialized. Please refresh the page.';
        console.error(error);
        if (window.notifyAuthError) {
            window.notifyAuthError(error);
        }
        return false;
    }

    try {
        googleAuth.prompt((notification) => {
            if (notification.isNotDisplayed() || notification.isSkippedMoment()) {
                console.log('Google One Tap not displayed or skipped');
                // Fallback to button click or other auth methods
            }
        });
        return true;
    } catch (error) {
        console.error('Failed to trigger login:', error);
        if (window.notifyAuthError) {
            window.notifyAuthError('Failed to start login flow: ' + error.message);
        }
        return false;
    }
}

/**
 * Clear stored authentication data
 */
function clearStoredAuthData() {
    try {
        localStorage.removeItem('cocomiro_google_id_token');
        localStorage.removeItem('cocomiro_user_info');
        localStorage.removeItem('cocomiro_auth_timestamp');

        sessionStorage.removeItem('cocomiro_google_id_token');
        sessionStorage.removeItem('cocomiro_user_info');
        sessionStorage.removeItem('cocomiro_auth_timestamp');

        currentUser = null;
        console.log('Authentication data cleared');
        return true;
    } catch (error) {
        console.error('Failed to clear auth data:', error);
        return false;
    }
}

/**
 * Check if user is currently authenticated
 */
function isAuthenticated() {
    try {
        const token = localStorage.getItem('cocomiro_google_id_token');
        if (!token) {
            return false;
        }

        // Validate token hasn't expired
        const payload = parseJwt(token);
        const currentTime = Date.now() / 1000;

        // Add 5-minute buffer before expiration
        return (payload.exp - 300) > currentTime;
    } catch (error) {
        console.error('Failed to validate authentication:', error);
        // Clear invalid data
        clearStoredAuthData();
        return false;
    }
}

/**
 * Get stored user information
 */
function getStoredUserInfo() {
    try {
        const userInfoStr = localStorage.getItem('cocomiro_user_info');
        if (!userInfoStr) {
            return null;
        }

        const userInfo = JSON.parse(userInfoStr);

        // Validate user info has required fields
        if (!userInfo.id || !userInfo.email || !userInfo.name) {
            console.warn('Stored user info is incomplete');
            return null;
        }

        return userInfo;
    } catch (error) {
        console.error('Failed to parse stored user info:', error);
        return null;
    }
}

/**
 * Get the stored authentication token
 */
function getStoredToken() {
    return localStorage.getItem('cocomiro_google_id_token');
}

/**
 * Parse a JWT token and return the payload
 */
function parseJwt(token) {
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
            return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
        }).join(''));

        return JSON.parse(jsonPayload);
    } catch (error) {
        console.error('Failed to parse JWT:', error);
        throw error;
    }
}

/**
 * Restore authentication state on page load
 */
function restoreAuthState() {
    if (isAuthenticated()) {
        currentUser = getStoredUserInfo();
        if (currentUser && window.notifyAuthRestored) {
            window.notifyAuthRestored(currentUser);
        }
        console.log('Authentication state restored for user:', currentUser?.name);
        return true;
    }

    console.log('No valid authentication state found');
    return false;
}

/**
 * Refresh the authentication token if needed
 */
function refreshTokenIfNeeded() {
    if (!isAuthenticated()) {
        console.log('Token refresh needed - user not authenticated');
        return false;
    }

    // For Google Identity Services, tokens are typically long-lived
    // and don't need manual refresh. The library handles this internally.
    // We could implement proactive refresh here if needed.

    return true;
}

/**
 * Initialize authentication when the page loads
 */
function initializeAuth() {
    // Wait for Google Identity Services to load
    if (window.google && window.google.accounts && window.google.accounts.id) {
        const success = initializeGoogleAuth();
        if (success) {
            // Try to restore previous session
            restoreAuthState();
        }
        return success;
    } else {
        // Retry after a short delay
        setTimeout(initializeAuth, 100);
        return false;
    }
}

// Export functions for use in other scripts and Rust interop
window.GoogleAuth = {
    initialize: initializeGoogleAuth,
    login: triggerGoogleLogin,
    logout: clearStoredAuthData,
    isAuthenticated: isAuthenticated,
    getUser: () => currentUser,
    getStoredUser: getStoredUserInfo,
    getToken: getStoredToken,
    restoreState: restoreAuthState,
    refreshToken: refreshTokenIfNeeded,
    clearData: clearStoredAuthData,
};

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeAuth);
} else {
    initializeAuth();
}