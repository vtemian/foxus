// Safely access Tauri API with fallback for browser dev mode
const invoke = window.__TAURI__?.core?.invoke;

// Default distraction budget for focus sessions (in minutes)
// TODO: Add UI to allow user to configure this value
const DEFAULT_BUDGET_MINUTES = 10;

if (!invoke) {
  console.warn("Tauri API not available - running in browser mode");
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function formatTime(secs) {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return `${hours}h ${mins}m`;
}

function formatBudget(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

async function loadStats() {
  if (!invoke) return;

  try {
    const stats = await invoke("get_today_stats");

    const total = stats.productive_secs + stats.neutral_secs + stats.distracting_secs;

    document.getElementById("bar-productive").style.width =
      total > 0 ? `${(stats.productive_secs / total) * 100}%` : "0%";
    document.getElementById("bar-neutral").style.width =
      total > 0 ? `${(stats.neutral_secs / total) * 100}%` : "0%";
    document.getElementById("bar-distracting").style.width =
      total > 0 ? `${(stats.distracting_secs / total) * 100}%` : "0%";

    document.getElementById("time-productive").textContent = formatTime(stats.productive_secs);
    document.getElementById("time-neutral").textContent = formatTime(stats.neutral_secs);
    document.getElementById("time-distracting").textContent = formatTime(stats.distracting_secs);

    const appList = document.getElementById("app-list");
    appList.innerHTML = stats.top_apps.map(app => {
      const prodClass = app.productivity > 0 ? "productive" :
                        app.productivity < 0 ? "distracting" : "neutral";
      const safeName = escapeHtml(app.name);
      return `
        <li>
          <span class="app-name">
            <span class="productivity-dot ${prodClass}"></span>
            ${safeName}
          </span>
          <span>${formatTime(app.duration_secs)}</span>
        </li>
      `;
    }).join("");
  } catch (e) {
    console.error("Failed to load stats:", e);
  }
}

async function loadFocusState() {
  if (!invoke) return;

  try {
    const state = await invoke("get_focus_state");
    const btn = document.getElementById("focus-btn");
    const statsView = document.getElementById("stats-view");
    const focusView = document.getElementById("focus-view");

    if (state.active) {
      statsView.style.display = "none";
      focusView.style.display = "block";
      btn.textContent = "End Focus Session";
      btn.classList.add("active");
      btn.setAttribute("aria-pressed", "true");
      document.getElementById("budget-time").textContent = formatBudget(state.budget_remaining);
    } else {
      statsView.style.display = "block";
      focusView.style.display = "none";
      btn.textContent = "Start Focus Session";
      btn.classList.remove("active");
      btn.setAttribute("aria-pressed", "false");
    }
  } catch (e) {
    console.error("Failed to load focus state:", e);
  }
}

async function toggleFocusSession() {
  if (!invoke) return;

  try {
    const state = await invoke("get_focus_state");
    if (state.active) {
      await invoke("end_focus_session");
    } else {
      await invoke("start_focus_session", { budgetMinutes: DEFAULT_BUDGET_MINUTES });
    }
    loadFocusState();
    loadStats();
  } catch (e) {
    console.error("Failed to toggle focus:", e);
  }
}

// Initialize when DOM is ready
document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("focus-btn").addEventListener("click", toggleFocusSession);

  // Initial load
  loadStats();
  loadFocusState();

  // Refresh periodically with guard against overlapping calls
  let refreshInProgress = false;
  setInterval(async () => {
    if (refreshInProgress) return;
    refreshInProgress = true;
    try {
      await Promise.all([loadStats(), loadFocusState()]);
    } finally {
      refreshInProgress = false;
    }
  }, 5000);
});
