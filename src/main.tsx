import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./hooks/useI18n";
import "./styles/tokens.css";
import "./styles/global.css";
import "./styles/markdown.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
    <I18nProvider>
        <App />
    </I18nProvider>,
);
