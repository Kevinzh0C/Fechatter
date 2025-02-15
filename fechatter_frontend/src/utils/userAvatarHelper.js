/**
 * User Avatar Helper
 * Generates consistent colors and initials for user avatars
 */

// Discord-style color palette - vibrant and distinct
const avatarColors = [
  '#f23f43', // Red
  '#f0b232', // Orange
  '#23a55a', // Green
  '#5865f2', // Blurple (Discord Blue)
  '#eb459e', // Pink
  '#3ba55c', // Dark Green
  '#faa61a', // Yellow
  '#ed4245', // Dark Red
  '#9146ff', // Purple (Twitch)
  '#00d4aa', // Cyan
  '#0099ff', // Sky Blue
  '#ff6b6b', // Light Red
  '#4ecdc4', // Teal
  '#f7b731', // Gold
  '#5f27cd', // Deep Purple
  '#00d2d3', // Turquoise
  '#ff9ff3', // Light Pink
  '#54a0ff', // Light Blue
  '#48dbfb', // Sky
  '#1dd1a1'  // Emerald
];

/**
 * Generate a consistent color for a user based on their ID
 * @param {number|string} userId - User ID or identifier
 * @returns {string} Hex color code
 */
export function getUserColor(userId) {
  if (!userId) return avatarColors[0];

  // Convert to number if string
  const id = typeof userId === 'string' ? parseInt(userId, 10) || 0 : userId;

  // Use modulo to get consistent color
  const index = Math.abs(id) % avatarColors.length;
  return avatarColors[index];
}

/**
 * Generate initials from user name
 * @param {string} fullname - User's full name
 * @param {string} fallback - Fallback if no name
 * @returns {string} 1-2 character initials
 */
export function getUserInitials(fullname, fallback = '?') {
  if (!fullname || typeof fullname !== 'string') return fallback;

  // Clean and split the name
  const cleanName = fullname.trim();
  if (!cleanName) return fallback;

  // Handle single word names
  const parts = cleanName.split(/\s+/);
  if (parts.length === 1) {
    return parts[0].substring(0, 2).toUpperCase();
  }

  // Get first letter of first and last name
  const firstInitial = parts[0][0] || '';
  const lastInitial = parts[parts.length - 1][0] || '';

  return (firstInitial + lastInitial).toUpperCase();
}

/**
 * Generate a gradient background for avatar
 * @param {number|string} userId - User ID
 * @returns {string} CSS gradient
 */
export function getUserGradient(userId) {
  const color1 = getUserColor(userId);
  const color2 = getUserColor(userId + 1); // Offset for second color

  return `linear-gradient(135deg, ${color1}, ${color2})`;
}

/**
 * Get contrast color (black or white) for text on colored background
 * @param {string} hexColor - Hex color code
 * @returns {string} 'black' or 'white'
 */
export function getContrastColor(hexColor) {
  // Remove # if present
  const hex = hexColor.replace('#', '');

  // Convert to RGB
  const r = parseInt(hex.substr(0, 2), 16);
  const g = parseInt(hex.substr(2, 2), 16);
  const b = parseInt(hex.substr(4, 2), 16);

  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  // Return black or white based on luminance
  return luminance > 0.5 ? '#000000' : '#ffffff';
}

/**
 * Create avatar data object
 * @param {Object} user - User object
 * @returns {Object} Avatar data
 */
export function createAvatarData(user) {
  if (!user) {
    return {
      color: avatarColors[0],
      initials: '?',
      textColor: '#ffffff'
    };
  }

  const color = getUserColor(user.id);
  const initials = getUserInitials(user.fullname || user.name || user.email);
  const textColor = getContrastColor(color);

  return {
    color,
    initials,
    textColor,
    gradient: getUserGradient(user.id),
    avatarUrl: user.avatar_url || null
  };
}

// Export for use in components
export default {
  getUserColor,
  getUserInitials,
  getUserGradient,
  getContrastColor,
  createAvatarData,
  avatarColors
}; 