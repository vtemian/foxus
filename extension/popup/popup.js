function formatTime(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

chrome.runtime.sendMessage({ type: "get_state" }, (state) => {
  const statusEl = document.getElementById("status");

  if (state && state.active) {
    statusEl.className = "status active";
    statusEl.innerHTML = `
      <div>Focus Mode Active</div>
      <div class="budget">${formatTime(state.budgetRemaining)} remaining</div>
    `;
  } else {
    statusEl.className = "status inactive";
    statusEl.innerHTML = `<div>Focus Mode Off</div>`;
  }
});
