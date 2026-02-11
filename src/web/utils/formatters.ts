/**
 * Format seconds as "Xh Ym" for display in stats.
 * @example formatTime(3665) // "1h 1m"
 */
export const formatTime = (secs: number): string => {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return `${hours}h ${mins}m`;
};

/**
 * Format seconds as "M:SS" for focus budget countdown.
 * @example formatBudget(125) // "2:05"
 */
export const formatBudget = (secs: number): string => {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
};

/**
 * Escape HTML special characters for safe DOM insertion.
 * @example escapeHtml("<script>") // "&lt;script&gt;"
 */
export const escapeHtml = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};
