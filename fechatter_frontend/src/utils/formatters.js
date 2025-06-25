/**
 * Utility functions for formatting various data types
 */

/**
 * Format timestamp for message display
 * @param {string|Date} timestamp - The timestamp to format
 * @returns {string} Formatted time string
 */
export function formatTimestamp(timestamp) {
  if (!timestamp) return ''

  const date = new Date(timestamp)
  const now = new Date()
  const diffInSeconds = Math.floor((now - date) / 1000)

  // Less than 1 minute ago
  if (diffInSeconds < 60) {
    return 'now'
  }

  // Less than 1 hour ago
  if (diffInSeconds < 3600) {
    const minutes = Math.floor(diffInSeconds / 60)
    return `${minutes}m ago`
  }

  // Less than 24 hours ago - show time only
  if (diffInSeconds < 86400) {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    })
  }

  // Less than 7 days ago - show time + relative day
  if (diffInSeconds < 604800) {
    const days = Math.floor(diffInSeconds / 86400)
    const timeString = date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    })
    return `${timeString} • ${days}d ago`
  }

  // More than 7 days ago - show time + date
  const isCurrentYear = date.getFullYear() === now.getFullYear()
  const timeString = date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })

  if (isCurrentYear) {
    const dateString = date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
    return `${timeString} • ${dateString}`
  } else {
    const dateString = date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
    return `${timeString} • ${dateString}`
  }
}

/**
 * Format file size in human readable format
 * @param {number} bytes - File size in bytes
 * @returns {string} Formatted file size
 */
export function formatFileSize(bytes) {
  if (!bytes || bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  const size = parseFloat((bytes / Math.pow(k, i)).toFixed(1))
  return `${size} ${sizes[i]}`
}

/**
 * Format number with commas
 * @param {number} num - Number to format
 * @returns {string} Formatted number
 */
export function formatNumber(num) {
  if (!num) return '0'
  return num.toLocaleString()
}

/**
 * Format duration in human readable format
 * @param {number} seconds - Duration in seconds
 * @returns {string} Formatted duration
 */
export function formatDuration(seconds) {
  if (!seconds || seconds === 0) return '0:00'

  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = Math.floor(seconds % 60)

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  } else {
    return `${minutes}:${secs.toString().padStart(2, '0')}`
  }
}

/**
 * Truncate text with ellipsis
 * @param {string} text - Text to truncate
 * @param {number} maxLength - Maximum length
 * @returns {string} Truncated text
 */
export function truncateText(text, maxLength = 100) {
  if (!text || text.length <= maxLength) return text
  return text.substring(0, maxLength).trim() + '...'
}

/**
 * Format user mention
 * @param {string} username - Username to format
 * @returns {string} Formatted mention
 */
export function formatMention(username) {
  return `@${username}`
}

/**
 * Format channel mention
 * @param {string} channelName - Channel name to format
 * @returns {string} Formatted channel mention
 */
export function formatChannelMention(channelName) {
  return `#${channelName}`
} 