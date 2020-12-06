import { extendTheme } from "@chakra-ui/react";
import React from "react";

const theme = extendTheme({
    textStyles: {
        actionable: {
            fontSize: "lg",
            color: "gray.500",
        },
        info: {
            fontSize: "sm",
            color: "gray.500",
        },
        assetSelect: {
            fontSize: "md",
            color: "gray.500",
        },
    },
    swapButton: {
        baseStyle: {
            colorScheme: "teal",
            size: "lg",
            bg: "#304FFE",
            rounded: "md",
            width: "200px",
            // _hover: {{ bg: "blue.300" }},
        },
    },
    components: {
        Button: {
            baseStyle: {
                bg: "#304FFE",
                fontWeight: "bold",
                fontColor: "green",
                color: "green",
                _hover: {
                    bg: "blue.300",
                },
            },

            sizes: {
                lg: {
                    h: "56px",
                    fontSize: "lg",
                    px: "32px",
                },
            },
            // Custom variant
            variants: {
                "main_button": {
                    h: "50px",
                    w: "300px",
                    color: "white",
                },
                "wallet_button": {
                    color: "white",
                },
            },
        },
    },
});

export default theme;
