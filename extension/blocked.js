const params = new URLSearchParams(window.location.search);
const blockedUrl = params.get("url");
const budget = parseInt(params.get("budget") || "0", 10);

// Validate URL to prevent open redirect attacks
function isValidHttpUrl(urlString) {
  if (!urlString) return false;
  try {
    const url = new URL(urlString);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

function getHostname(urlString) {
  try {
    return new URL(urlString).hostname;
  } catch {
    return "Unknown site";
  }
}

const validUrl = isValidHttpUrl(blockedUrl);
document.getElementById("domain").textContent = validUrl ? getHostname(blockedUrl) : "Unknown site";

function formatTime(secs) {
  // Ensure input is a valid number to prevent issues with non-numeric input
  const safeSecs = Math.max(0, Math.floor(Number(secs) || 0));
  const mins = Math.floor(safeSecs / 60);
  const s = safeSecs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

document.getElementById("budget-time").textContent = formatTime(budget);

if (budget <= 0 || !validUrl) {
  document.getElementById("soft-block").style.display = "none";
  document.getElementById("hard-block").style.display = "block";
} else {
  const btn = document.getElementById("use-budget-btn");
  const countdown = document.getElementById("countdown");
  const countdownTime = document.getElementById("countdown-time");

  btn.addEventListener("click", () => {
    btn.disabled = true;
    countdown.style.display = "block";

    let remaining = 30;
    countdownTime.textContent = remaining;

    const interval = setInterval(() => {
      remaining--;
      countdownTime.textContent = remaining;

      if (remaining <= 0) {
        clearInterval(interval);
        chrome.runtime.sendMessage({ type: "use_distraction_time" }, () => {
          // Only redirect to validated HTTP(S) URLs to prevent open redirect
          if (validUrl) {
            window.location.href = blockedUrl;
          }
        });
      }
    }, 1000);
  });
}
