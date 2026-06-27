import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { UiScaleProvider } from "./hooks/useUiScale";
import { ThemeProvider } from "./hooks/useTheme";
import { applyTheme, readStoredTheme } from "./lib/theme";
import { applyUiScale, readStoredUiScale } from "./lib/uiScale";
import "./index.css";

applyTheme(readStoredTheme());
void applyUiScale(readStoredUiScale());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <UiScaleProvider>
        <App />
      </UiScaleProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
