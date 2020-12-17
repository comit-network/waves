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
        addressInfo: {
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
            },

            sizes: {
                lg: {
                    fontSize: "lg",
                },
            },
            // Custom variant
            variants: {
                "tx_button": {
                    bg: "blue.100",
                    color: "white",
                },
                "main_button": {
                    w: "15rem",
                    color: "white",
                    _hover: {
                        bg: "blue.300",
                    },
                },
                "wallet_button": {
                    color: "white",
                    _hover: {
                        bg: "blue.300",
                    },
                },
            },
        },
    },
});

export default theme;
