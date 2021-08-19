import { Box, Button, Heading } from "@chakra-ui/react";
import React from "react";
import { useAsync } from "react-async";
import { backgroundPage, Trade } from "../background/api";
import YouSwapItem from "./SwapItem";

interface ConfirmSwapProps {
    onCancel: () => void;
    onSuccess: () => void;
    trade: Trade;
}

export default function ConfirmSwap(
    { onCancel, onSuccess, trade }: ConfirmSwapProps,
) {
    let { isPending, run } = useAsync({
        deferFn: async () => {
            const page = await backgroundPage();
            await page.approveSwap();

            onSuccess();
        },
    });

    return (<Box>
        <form
            onSubmit={e => {
                e.preventDefault();
                run();
            }}
        >
            <Heading>Confirm Swap</Heading>
            <Box>
                <YouSwapItem
                    tradeSide={trade.sell}
                    action={"send"}
                />
            </Box>
            <Box>
                <YouSwapItem
                    tradeSide={trade.buy}
                    action={"receive"}
                />
            </Box>

            <Button
                variant="secondary"
                mr={3}
                onClick={async () => {
                    const page = await backgroundPage();
                    await page.rejectSwap();

                    onCancel();
                }}
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
