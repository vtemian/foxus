const params = new URLSearchParams(window.location.search);
const blockedUrl = params.get("url");
const budget = parseInt(params.get("budget") || "0", 10);

document.getElementById("domain").textContent = blockedUrl ? new URL(blockedUrl).hostname : "Unknown site";

function formatTime(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

document.getElementById("budget-time").textContent = formatTime(budget);

if (budget <= 0) {
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
          window.location.href = blockedUrl;
        });
      }
    }, 1000);
  });
}
