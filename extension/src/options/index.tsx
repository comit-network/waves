import { ChakraProvider } from "@chakra-ui/react";
import React from "react";
import ReactDOM from "react-dom";
import theme from "../theme";
import Options from "./Options";

ReactDOM.render(
    <React.StrictMode>
        <ChakraProvider theme={theme}>
            <Options />
        </ChakraProvider>
    </React.StrictMode>,
    document.getElementById("options"),
);
