const NATIVE_HOST = "com.foxus.native";

let focusState = {
  active: false,
  budgetRemaining: 0,
  blockedDomains: []
};

let nativePort = null;

function connectToNative() {
  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST);

    nativePort.onMessage.addListener((message) => {
      console.log("Native message:", message);
      if (message.type === "state") {
        focusState = {
          active: message.focusActive,
          budgetRemaining: message.budgetRemaining,
          blockedDomains: message.blockedDomains || []
        };
        chrome.storage.local.set({ focusState });
      } else if (message.type === "budget_updated") {
        focusState.budgetRemaining = message.remaining;
        chrome.storage.local.set({ focusState });
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log("Native host disconnected");
      nativePort = null;
      // Retry connection after delay
      setTimeout(connectToNative, 5000);
    });

    // Request initial state
    nativePort.postMessage({ type: "request_state" });
  } catch (e) {
    console.error("Failed to connect to native host:", e);
    // Load cached state
    chrome.storage.local.get(["focusState"], (result) => {
      if (result.focusState) {
        focusState = result.focusState;
      }
    });
  }
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
  const tab = await chrome.tabs.get(activeInfo.tabId);
  if (tab.url) {
    sendActivity(tab.url, tab.title || "");
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
