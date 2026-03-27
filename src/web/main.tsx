import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "@/App";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Root element not found. Add <div id='root'></div> to index.html");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
