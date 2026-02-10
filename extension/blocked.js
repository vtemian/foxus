import { formatTime, isValidHttpUrl, getHostname } from './utils.js';

const params = new URLSearchParams(window.location.search);
const blockedUrl = params.get("url");
const budget = Math.max(0, Math.floor(Number(params.get("budget")) || 0));

const validUrl = isValidHttpUrl(blockedUrl);
document.getElementById("domain").textContent = validUrl ? getHostname(blockedUrl) : "Unknown site";

document.getElementById("budget-time").textContent = formatTime(budget);

if (budget <= 0 || !validUrl) {
  document.getElementById("soft-block").style.display = "none";
  document.getElementById("hard-block").style.display = "block";
} else {
  const btn = document.getElementById("use-budget-btn");
  const countdown = document.getElementById("countdown");
  const countdownTime = document.getElementById("countdown-time");
  let countdownInterval = null;

  btn.addEventListener("click", () => {
    // Clear any existing countdown to prevent stacking
    if (countdownInterval) {
      clearInterval(countdownInterval);
    }

    btn.disabled = true;
    countdown.style.display = "block";

    let remaining = 30;
    countdownTime.textContent = remaining;

    countdownInterval = setInterval(() => {
      remaining--;
      countdownTime.textContent = remaining;

      if (remaining <= 0) {
        clearInterval(countdownInterval);
        countdownInterval = null;
        chrome.runtime.sendMessage({ type: "use_distraction_time" }, () => {
          // Only redirect to validated HTTP(S) URLs to prevent open redirect
          if (validUrl) {
            window.location.href = blockedUrl;
          }
        });
      }
    }, 1000);
  });

  // Clean up interval on page unload
  window.addEventListener("beforeunload", () => {
    if (countdownInterval) {
      clearInterval(countdownInterval);
    }
  });
}
