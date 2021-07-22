import React from "react";
import ReactDOM from "react-dom";
import "../index.css";
import App from "./App";

function checkLogger() {
    const debugSettings = localStorage.getItem("debug");
    if (debugSettings) {
        // `debug` var set: we honor existing settings and do not overwrite them.
        return;
    }

    if (process.env.NODE_ENV === "production") {
        // if `debug` variable is not set and we are in production mode: give a warning that user won't see logs.
        // eslint-disable-next-line no-console
        console.log("`debug` variable not set. You won't see any logs unless you add `debug=*` to your localStorage.");
    } else if (process.env.NODE_ENV === "development") {
        // if `debug` variable is not set but we are in development mode: enable all logs.
        localStorage.setItem("debug", "*");
    }
}

checkLogger();

ReactDOM.render(
    <React.StrictMode>
        <App />
    </React.StrictMode>,
    document.getElementById("root"),
);
