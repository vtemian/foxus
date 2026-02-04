const { invoke } = window.__TAURI__.core;

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
  try {
    const stats = await invoke("get_today_stats");

    const total = stats.productive_secs + stats.neutral_secs + stats.distracting_secs;
    const maxPercent = total > 0 ? 100 : 0;

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
      return `
        <li>
          <span class="app-name">
            <span class="productivity-dot ${prodClass}"></span>
            ${app.name}
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
      document.getElementById("budget-time").textContent = formatBudget(state.budget_remaining);
    } else {
      statsView.style.display = "block";
      focusView.style.display = "none";
      btn.textContent = "Start Focus Session";
      btn.classList.remove("active");
    }
  } catch (e) {
    console.error("Failed to load focus state:", e);
  }
}

document.getElementById("focus-btn").addEventListener("click", async () => {
  try {
    const state = await invoke("get_focus_state");
    if (state.active) {
      await invoke("end_focus_session");
    } else {
      await invoke("start_focus_session", { budgetMinutes: 10 });
    }
    loadFocusState();
    loadStats();
  } catch (e) {
    console.error("Failed to toggle focus:", e);
  }
});

// Initial load
loadStats();
loadFocusState();

// Refresh periodically
setInterval(() => {
  loadStats();
  loadFocusState();
}, 5000);
