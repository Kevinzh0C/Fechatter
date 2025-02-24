/**
 * User Avatar Helper
 *
 * Provides utility functions for generating user initials and consistent,
 * unique colors for avatars based on user identifiers. This ensures that
 * users without a profile picture still have a recognizable and visually
 * distinct representation.
 */

/**
 * Generates initials from a user's full name.
 * @param {string} name The full name of the user.
 * @returns {string} A 1 or 2 character uppercase string representing the initials.
 */
export function getUserInitials(name) {
  if (!name || typeof name !== 'string') {
    return '?';
  }

  const parts = name.trim().split(/\s+/);
  if (parts.length === 1) {
    return parts[0].charAt(0).toUpperCase();
  }

  const firstInitial = parts[0].charAt(0);
  const lastInitial = parts[parts.length - 1].charAt(0);

  return `${firstInitial}${lastInitial}`.toUpperCase();
}

/**
 * A predefined, aesthetically pleasing color palette for avatar backgrounds.
 * These colors are chosen to have good contrast with white text.
 */
const avatarColors = [
  '#ef4444', // red-500
  '#f97316', // orange-500
  '#eab308', // yellow-500
  '#84cc16', // lime-500
  '#22c55e', // green-500
  '#10b981', // emerald-500
  '#14b8a6', // teal-500
  '#06b6d4', // cyan-500
  '#3b82f6', // blue-500
  '#6366f1', // indigo-500
  '#8b5cf6', // violet-500
  '#d946ef', // fuchsia-500
  '#ec4899', // pink-500
];

/**
 * This strategy ensures that foundational colors (like blue, red, black) are present,
 * while still providing unique shades for each user through hue shifting.
 */
const baseColors = [
  { h: 210, s: 85, l: 55 }, // Vibrant Blue
  { h: 0, s: 80, l: 50 },   // Strong Red
  { h: 145, s: 65, l: 45 }, // Emerald Green
  { h: 35, s: 90, l: 55 },  // Sunny Orange
  { h: 260, s: 70, l: 60 }, // Deep Purple
  { h: 185, s: 75, l: 40 }, // Teal
  { h: 330, s: 85, l: 60 }, // Magenta Pink
  // Add a near-black, but with a hint of color for personality.
  { h: 220, s: 15, l: 25 }, // Graphite (very dark, slightly blue)
];

/**
 * Generates a consistent, aesthetically pleasing color from a user ID.
 * It selects a base color and applies a slight, deterministic hue shift.
 *
 * @param {string | number} id The unique identifier for the user.
 * @returns {string} An HSL color string.
 */
export function generateAvatarColor(id) {
  if (id === null || id === undefined) {
    return 'hsl(220, 15%, 25%)'; // Neutral Graphite fallback
  }

  const strId = String(id);
  let hash = 0;
  for (let i = 0; i < strId.length; i++) {
    hash = strId.charCodeAt(i) + ((hash << 5) - hash);
  }
  hash = Math.abs(hash);

  // Select a base color from the palette
  const baseColor = baseColors[hash % baseColors.length];

  // Apply a small, deterministic hue shift for uniqueness
  // The shift is between -15 and +15 degrees
  const hueShift = (hash % 31) - 15;
  const finalHue = (baseColor.h + hueShift + 360) % 360;

  return `hsl(${finalHue}, ${baseColor.s}%, ${baseColor.l}%)`;
}

/**
 * Generate a gradient background for avatar
 * @param {number|string} userId - User ID
 * @returns {string} CSS gradient
 */
export function getUserGradient(userId) {
  const color1 = generateAvatarColor(userId);
  const color2 = generateAvatarColor(userId + 1); // Offset for second color

  return `linear-gradient(135deg, ${color1}, ${color2})`;
}

/**
 * Get contrast color (black or white) for text on colored background
 * @param {string} colorString - Hex or HSL color string (e.g., '#RRGGBB' or 'hsl(H, S%, L%)').
 * @returns {string} 'black' or 'white'
 */
export function getContrastColor(colorString) {
  let r, g, b;

  if (colorString.startsWith('#')) {
    // Handle HEX
    const hex = colorString.replace('#', '');
    r = parseInt(hex.substr(0, 2), 16);
    g = parseInt(hex.substr(2, 2), 16);
    b = parseInt(hex.substr(4, 2), 16);
  } else if (colorString.startsWith('hsl')) {
    // Handle HSL
    const [h, s, l] = colorString.match(/\d+/g).map(Number);
    [r, g, b] = hslToRgb(h, s / 100, l / 100);
  } else {
    // Fallback for unknown formats
    return '#ffffff';
  }

  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  // Return black or white based on luminance
  return luminance > 0.5 ? '#000000' : '#ffffff';
}

/**
 * Converts an HSL color value to RGB.
 * @param {number} h The hue
 * @param {number} s The saturation
 * @param {number} l The lightness
 * @returns {number[]} The RGB representation
 */
function hslToRgb(h, s, l) {
  let r, g, b;

  if (s === 0) {
    r = g = b = l; // achromatic
  } else {
    const hue2rgb = (p, q, t) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h / 360 + 1 / 3);
    g = hue2rgb(p, q, h / 360);
    b = hue2rgb(p, q, h / 360 - 1 / 3);
  }

  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
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

  const color = generateAvatarColor(user.id);
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
  getUserInitials,
  generateAvatarColor,
  getUserGradient,
  getContrastColor,
  createAvatarData,
  avatarColors
}; 