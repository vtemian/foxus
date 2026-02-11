import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Root element not found. Add <div id='root'></div> to index.html");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
