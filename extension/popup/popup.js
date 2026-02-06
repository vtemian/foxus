function formatTime(secs) {
  // Ensure input is a valid number to prevent injection
  const safeSecs = Math.max(0, Math.floor(Number(secs) || 0));
  const mins = Math.floor(safeSecs / 60);
  const s = safeSecs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

function createStatusContent(isActive, budgetRemaining) {
  const container = document.createDocumentFragment();

  if (isActive) {
    const statusDiv = document.createElement("div");
    statusDiv.textContent = "Focus Mode Active";
    container.appendChild(statusDiv);

    const budgetDiv = document.createElement("div");
    budgetDiv.className = "budget";
    budgetDiv.textContent = `${formatTime(budgetRemaining)} remaining`;
    container.appendChild(budgetDiv);
  } else {
    const statusDiv = document.createElement("div");
    statusDiv.textContent = "Focus Mode Off";
    container.appendChild(statusDiv);
  }

  return container;
}

chrome.runtime.sendMessage({ type: "get_state" }, (state) => {
  const statusEl = document.getElementById("status");

  if (state && state.active) {
    statusEl.className = "status active";
    statusEl.replaceChildren(createStatusContent(true, state.budgetRemaining));
  } else {
    statusEl.className = "status inactive";
    statusEl.replaceChildren(createStatusContent(false, 0));
  }
});
