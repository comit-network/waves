import { ChakraProvider } from "@chakra-ui/react";
import theme from "../theme";

const App = () => {
    return (
        <ChakraProvider theme={theme}>
            <p>Welcome to the Extension Page</p>
        </ChakraProvider>
    );
};

export default App;
