const NATIVE_HOST = "com.foxus.native";
const MAX_RECONNECT_ATTEMPTS = 10;
const INITIAL_RECONNECT_DELAY = 1000;

let focusState = {
  active: false,
  budgetRemaining: 0,
  blockedDomains: []
};

let nativePort = null;
let reconnectAttempts = 0;
let reconnectTimeoutId = null;

function validateStateMessage(message) {
  return (
    message &&
    typeof message === "object" &&
    typeof message.focusActive === "boolean" &&
    typeof message.budgetRemaining === "number" &&
    (message.blockedDomains === undefined || Array.isArray(message.blockedDomains))
  );
}

function validateBudgetMessage(message) {
  return (
    message &&
    typeof message === "object" &&
    typeof message.remaining === "number"
  );
}

function connectToNative() {
  if (reconnectTimeoutId) {
    clearTimeout(reconnectTimeoutId);
    reconnectTimeoutId = null;
  }

  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST);
    reconnectAttempts = 0;

    nativePort.onMessage.addListener((message) => {
      console.log("Native message:", message);
      if (message && message.type === "state" && validateStateMessage(message)) {
        focusState = {
          active: message.focusActive,
          budgetRemaining: message.budgetRemaining,
          blockedDomains: message.blockedDomains || []
        };
        chrome.storage.local.set({ focusState });
      } else if (message && message.type === "budget_updated" && validateBudgetMessage(message)) {
        focusState.budgetRemaining = message.remaining;
        chrome.storage.local.set({ focusState });
      } else if (message && message.type) {
        console.warn("Unhandled or invalid message type:", message.type);
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log("Native host disconnected");
      nativePort = null;
      scheduleReconnect();
    });

    // Request initial state
    nativePort.postMessage({ type: "request_state" });
  } catch (e) {
    console.error("Failed to connect to native host:", e);
    loadCachedState();
    scheduleReconnect();
  }
}

function scheduleReconnect() {
  if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
    console.warn("Max reconnect attempts reached. Loading cached state.");
    loadCachedState();
    return;
  }

  const delay = INITIAL_RECONNECT_DELAY * Math.pow(2, reconnectAttempts);
  reconnectAttempts++;
  console.log(`Scheduling reconnect attempt ${reconnectAttempts} in ${delay}ms`);
  reconnectTimeoutId = setTimeout(connectToNative, delay);
}

function loadCachedState() {
  chrome.storage.local.get(["focusState"], (result) => {
    if (result.focusState) {
      focusState = result.focusState;
    }
  });
}

function sendActivity(url, title) {
  if (nativePort) {
    nativePort.postMessage({
      type: "activity",
      url,
      title,
      timestamp: Date.now()
    });
  }
}

function isDomainBlocked(url) {
  if (!focusState.active) return false;

  try {
    const domain = new URL(url).hostname;
    return focusState.blockedDomains.some(blocked => {
      if (blocked.startsWith("*.")) {
        return domain.endsWith(blocked.slice(1));
      }
      return domain === blocked || domain.endsWith("." + blocked);
    });
  } catch {
    return false;
  }
}

// Track tab changes
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  try {
    const tab = await chrome.tabs.get(activeInfo.tabId);
    if (tab && tab.url) {
      sendActivity(tab.url, tab.title || "");
    }
  } catch (e) {
    // Tab may not exist or be accessible (e.g., chrome:// pages)
    console.debug("Could not get tab info:", e.message);
  }
});

// Track URL changes
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.url) {
    sendActivity(changeInfo.url, tab.title || "");
  }
});

// Block navigation to distracting sites
chrome.webNavigation.onBeforeNavigate.addListener((details) => {
  if (details.frameId !== 0) return; // Only main frame

  if (isDomainBlocked(details.url)) {
    const blockedUrl = chrome.runtime.getURL("blocked.html") +
      "?url=" + encodeURIComponent(details.url) +
      "&budget=" + focusState.budgetRemaining;

    chrome.tabs.update(details.tabId, { url: blockedUrl });
  }
});

// Handle messages from popup and blocked page
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "get_state") {
    sendResponse(focusState);
  } else if (message.type === "use_distraction_time") {
    if (nativePort) {
      nativePort.postMessage({ type: "use_distraction_time" });
    }
    sendResponse({ success: true });
  }
  return true;
});

// Initialize
connectToNative();
