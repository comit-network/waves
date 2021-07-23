import { ChakraProvider } from "@chakra-ui/react";
import theme from "../theme";

import {
    getBalances
} from "./background-proxy";
import {useEffect, useState} from "react";

// TODO: Should we just reuse the model or does it make sense to create a new one so the extension page is independent form the other code?
// I reckon reuse is fine - we could potentially even use the same background proxy - or create one that is shared between components...?
import {BalanceUpdate} from "../models";

const App = () => {
    let [balance, setBalance] = useState<null | BalanceUpdate>(null);

    // TODO: use-effect does not make much sense here, this is just for validating that this works as expected
    useEffect(() => {
        const fetchBalance = async () => {
            const bu = await getBalances();
            setBalance(bu);
        }

        if (!balance) {
            fetchBalance()
        }
    }, []);

    return (
        <ChakraProvider theme={theme}>
            <p>Welcome to the Extension Page {JSON.stringify(balance)}</p>
        </ChakraProvider>
    );
};

export default App;
