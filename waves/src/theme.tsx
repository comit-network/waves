import { extendTheme } from "@chakra-ui/react";

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
                    fontSize: "lg",
                },
            },
            // Custom variant
            variants: {
                "main_button": {
                    w: "15rem",
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
