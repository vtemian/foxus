const SECONDS_PER_HOUR = 3600;
const SECONDS_PER_MINUTE = 60;

/**
 * Format seconds as "Xh Ym" for display in stats.
 * @example formatTime(3665) // "1h 1m"
 */
const formatTime = (secs: number): string => {
  const hours = Math.floor(secs / SECONDS_PER_HOUR);
  const mins = Math.floor((secs % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  return `${hours}h ${mins}m`;
};

/**
 * Format seconds as "M:SS" for focus budget countdown.
 * @example formatBudget(125) // "2:05"
 */
const formatBudget = (secs: number): string => {
  const mins = Math.floor(secs / SECONDS_PER_MINUTE);
  const s = secs % SECONDS_PER_MINUTE;
  return `${mins}:${s.toString().padStart(2, "0")}`;
};

/**
 * Escape HTML special characters for safe DOM insertion.
 * @example escapeHtml("<script>") // "&lt;script&gt;"
 */
const escapeHtml = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};

export { escapeHtml, formatBudget, formatTime };
