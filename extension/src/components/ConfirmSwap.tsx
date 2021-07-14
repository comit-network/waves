import { Box, Button, Heading } from "@chakra-ui/react";
import React from "react";
import { useAsync } from "react-async";
import { signAndSendSwap } from "../background-proxy";
import { SwapToSign } from "../models";
import YouSwapItem from "./SwapItem";

interface ConfirmSwapProps {
    onCancel: (tabId: number) => void;
    onSuccess: () => void;
    swapToSign: SwapToSign;
}

export default function ConfirmSwap(
    { onCancel, onSuccess, swapToSign }: ConfirmSwapProps,
) {
    let { isPending, run } = useAsync({
        deferFn: async () => {
            await signAndSendSwap(swapToSign.txHex, swapToSign.tabId);
            onSuccess();
        },
    });

    let { decoded } = swapToSign;

    return (<Box>
        <form
            onSubmit={async e => {
                e.preventDefault();
                run();
            }}
        >
            <Heading>Confirm Swap</Heading>
            <Box>
                <YouSwapItem
                    tradeSide={decoded.sell}
                    action={"send"}
                />
            </Box>
            <Box>
                <YouSwapItem
                    tradeSide={decoded.buy}
                    action={"receive"}
                />
            </Box>

            <Button
                variant="secondary"
                mr={3}
                onClick={() => onCancel(swapToSign.tabId)}
            >
                Cancel
            </Button>
            <Button
                type="submit"
                variant="primary"
                isLoading={isPending}
                data-cy="data-cy-sign-and-send-button"
            >
                Sign and send
            </Button>
        </form>
    </Box>);
}
