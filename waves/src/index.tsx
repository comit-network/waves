import { ChakraProvider } from "@chakra-ui/react";
import Debug from "debug";
import React from "react";
import ReactDOM from "react-dom";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { BobtimusRateProvider } from "./Bobtimus";
import "./index.css";
import reportWebVitals from "./reportWebVitals";
import theme from "./theme";
const webVitalsLogger = Debug("webVitals");

function checkLogger() {
    const debugSettings = localStorage.getItem("debug");
    if (debugSettings) {
        // `debug` var set: we honor existing settings and do not overwrite them.
        return;
    }

    if (process.env.NODE_ENV === "production") {
        // if `debug` variable is not set and we are in production mode: give a warning that user won't see logs.
        console.log("`debug` variable not set. You won't see any logs unless you add `debug=*` to your localStorage.");
    } else if (process.env.NODE_ENV === "development") {
        // if `debug` variable is not set but we are in development mode: enable all logs.
        localStorage.setItem("debug", "*");
    }
}

checkLogger();

ReactDOM.render(
    <React.StrictMode>
        <ChakraProvider theme={theme}>
            <BobtimusRateProvider>
                <BrowserRouter>
                    <App />
                </BrowserRouter>
            </BobtimusRateProvider>
        </ChakraProvider>
    </React.StrictMode>,
    document.getElementById("root"),
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals(webVitalsLogger);
