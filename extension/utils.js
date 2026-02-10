/**
 * Format seconds as MM:SS string.
 * @param {number} secs - Number of seconds
 * @returns {string} Formatted time string
 */
export function formatTime(secs) {
  // Ensure input is a valid number to prevent injection
  const safeSecs = Math.max(0, Math.floor(Number(secs) || 0));
  const mins = Math.floor(safeSecs / 60);
  const s = safeSecs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

/**
 * Validate that a URL is a valid HTTP(S) URL.
 * Used to prevent open redirect attacks.
 * @param {string} urlString - URL to validate
 * @returns {boolean} True if valid HTTP(S) URL
 */
export function isValidHttpUrl(urlString) {
  if (!urlString) return false;
  try {
    const url = new URL(urlString);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

/**
 * Extract hostname from a URL string.
 * @param {string} urlString - URL to parse
 * @returns {string} Hostname or "Unknown site" on error
 */
export function getHostname(urlString) {
  try {
    return new URL(urlString).hostname;
  } catch {
    return "Unknown site";
  }
}
