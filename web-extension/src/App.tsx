import { Box, ChakraProvider, Code, Grid, Link, Text, theme, VStack } from "@chakra-ui/react";
import * as React from "react";
import { browser } from "webextension-polyfill-ts";
import { Logo } from "./Logo";

const App = () => {
    const bgPage = browser.extension.getBackgroundPage();
    // @ts-ignore because we know it's there :)
    const response = bgPage.someMethodInBGPage();

    return (<ChakraProvider theme={theme}>
        <Box textAlign="center" fontSize="xl">
            <Grid minH="100vh" p={3}>
                <VStack spacing={8}>
                    <Logo h="40vmin" pointerEvents="none" />
                    <Text>
                        Edit <Code fontSize="xl">src/App.tsx</Code> and save to reload.
                    </Text>
                    <Link
                        color="teal.500"
                        href="https://chakra-ui.com"
                        fontSize="2xl"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        COMIT Waves 2.0 says {response}
                    </Link>
                </VStack>
            </Grid>
        </Box>
    </ChakraProvider>);
};

export default App;
