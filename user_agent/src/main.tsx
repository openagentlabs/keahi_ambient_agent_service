import * as React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

console.log("React app starting...");

const container = document.getElementById("root");
if (container) {
  console.log("Root container found, creating React app...");
  const root = createRoot(container);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  console.log("React app rendered successfully");
} else {
  console.error("Root container not found!");
  // Fallback: create the root element if it doesn't exist
  const newContainer = document.createElement("div");
  newContainer.id = "root";
  document.body.appendChild(newContainer);
  const root = createRoot(newContainer);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}
