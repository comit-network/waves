import { ChakraProvider } from "@chakra-ui/react";
import React from "react";
import ReactDOM from "react-dom";
import App from "./App";
import { Provider as RateServiceProvider } from "./hooks/RateService";
import "./index.css";
import reportWebVitals from "./reportWebVitals";
import theme from "./theme";

ReactDOM.render(
    <React.StrictMode>
        <ChakraProvider theme={theme}>
            <RateServiceProvider value="http://getbestrate.com">
                <App />
            </RateServiceProvider>
        </ChakraProvider>
    </React.StrictMode>,
    document.getElementById("root"),
);

import("./wallet/pkg").then(wallet => (window as any).window = wallet);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals(console.log);
