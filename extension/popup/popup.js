import { formatTime } from '../utils.js';

function updateStatus(isActive, budgetRemaining) {
  const statusEl = document.getElementById("status");
  const statusValueEl = document.getElementById("status-value");
  const budgetEl = document.getElementById("budget");
  const budgetTimeEl = document.getElementById("budget-time");

  if (!statusEl || !statusValueEl) {
    console.error("Required elements not found in popup.html");
    return;
  }

  statusValueEl.classList.remove("loading");

  if (isActive) {
    statusEl.className = "status active";
    statusValueEl.textContent = "Active";
    if (budgetEl && budgetTimeEl) {
      budgetEl.style.display = "block";
      budgetTimeEl.textContent = formatTime(budgetRemaining);
    }
  } else {
    statusEl.className = "status inactive";
    statusValueEl.textContent = "Inactive";
    if (budgetEl) {
      budgetEl.style.display = "none";
    }
  }
}

chrome.runtime.sendMessage({ type: "get_state" }, (state) => {
  if (state && state.active) {
    updateStatus(true, state.budgetRemaining);
  } else {
    updateStatus(false, 0);
  }
});
