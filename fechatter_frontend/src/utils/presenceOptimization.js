/**
 * Presence Optimization Documentation
 * 
 * Problem: Users were being marked as "away" too aggressively
 * - Tab switch → immediate away status  
 * - Window blur → 1 second delay → away status
 * 
 * Solution (Occam's Razor): Disable auto-away completely
 * - Users stay online unless they disconnect or manually set status
 * - This matches modern chat app behavior (Slack, Discord, etc.)
 * 
 * To re-enable auto-away in the future:
 * 1. Set developmentOptimizer.presence.autoAwayEnabled = true
 * 2. Uncomment the code in sse.js handleVisibilityChange/handleWindowBlur
 */

import developmentOptimizer from './developmentOptimizer';

export function getPresenceConfig() {
  return developmentOptimizer.getConfig('presence') || {
    autoAwayEnabled: false,
    autoAwayDelayMinutes: 10,
    detectTabSwitch: false,
    detectWindowBlur: false,
    keepAliveOnFocus: true
  };
}

export function enableAutoAway() {
  if (import.meta.env.DEV) {
    console.warn('⚠️ Auto-away is currently disabled by design');
  if (import.meta.env.DEV) {
    console.log('To enable, uncomment code in:');
  if (import.meta.env.DEV) {
    console.log('- src/services/sse.js → handleVisibilityChange()');
  if (import.meta.env.DEV) {
    console.log('- src/services/sse.js → handleWindowBlur()');
  if (import.meta.env.DEV) {
    console.log('- src/services/sse-enhanced.js → same methods');
  }

// Expose for debugging
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.presenceConfig = {
    get: getPresenceConfig,
    enableAutoAway
  };

  if (import.meta.env.DEV) {
    console.log('👁️ Presence optimization active');
  if (import.meta.env.DEV) {
    console.log('Auto-away is DISABLED - users stay online');
  if (import.meta.env.DEV) {
    console.log('Use window.presenceConfig for settings');
  }

export default { getPresenceConfig, enableAutoAway }; 