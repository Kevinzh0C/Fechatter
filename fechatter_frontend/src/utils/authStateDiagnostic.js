/**
 * Authentication State Diagnostic
 * Simple utility to diagnose auth state consistency
 * Following Occam's Razor: Token is the source of truth
 */

import { useAuthStore } from '@/stores/auth';
import tokenManager from '@/services/tokenManager';

export function diagnoseAuthState() {
  console.group('🔍 Auth State Diagnostic');

  const authStore = useAuthStore();

  // Get all relevant states
  const states = {
    // Store state
    isAuthenticated: authStore.isAuthenticated,
    isLoggedIn: authStore.isLoggedIn,
    hasUser: !!authStore.user,
    userEmail: authStore.user?.email,

    // Token state
    hasToken: !!tokenManager.getAccessToken(),
    isTokenExpired: tokenManager.isTokenExpired(),
    tokenPreview: tokenManager.getAccessToken()?.substring(0, 20) + '...' || 'NULL',

    // Storage state
    hasStoredAuth: !!localStorage.getItem('auth')
  };

  // Log states
  console.table(states);

  // Check consistency
  const issues = [];

  // Rule 1: No token = Not authenticated
  if (states.isAuthenticated && !states.hasToken) {
    issues.push('❌ CRITICAL: isAuthenticated=true but no token exists');
  }

  // Rule 2: isLoggedIn should reflect token state
  if (states.isLoggedIn && !states.hasToken) {
    issues.push('❌ CRITICAL: isLoggedIn=true but no token exists');
  }

  // Rule 3: Expired token = Not logged in
  if (states.isLoggedIn && states.isTokenExpired) {
    issues.push('⚠️ WARNING: isLoggedIn=true but token is expired');
  }

  // Rule 4: User without token is inconsistent
  if (states.hasUser && !states.hasToken && states.isAuthenticated) {
    issues.push('⚠️ WARNING: User data exists without valid token');
  }

  // Report results
  if (issues.length === 0) {
    console.log('✅ Auth state is consistent');
    console.log('🔐 Token is the source of truth for authentication');
  } else {
    console.warn('🚨 Auth state inconsistencies detected:');
    issues.forEach(issue => console.warn(issue));
    console.log('\n💡 Solution: Clear auth and login again');
    console.log('Run: authStore.clearAuth()');
  }

  console.groupEnd();

  return {
    states,
    issues,
    isConsistent: issues.length === 0
  };
}

// Auto-diagnose on auth errors in development
if (import.meta.env.DEV && typeof window !== 'undefined') {
  window.addEventListener('error', (event) => {
    if (event.error?.message?.includes('Token missing') ||
      event.error?.message?.includes('authentication')) {
      console.warn('🔍 Auto-diagnosing auth state due to error...');
      diagnoseAuthState();
    }
  });
}

// Expose for debugging
if (typeof window !== 'undefined') {
  window.diagnoseAuthState = diagnoseAuthState;
  console.log('🔍 Auth diagnostic available: window.diagnoseAuthState()');
}

export default diagnoseAuthState; 