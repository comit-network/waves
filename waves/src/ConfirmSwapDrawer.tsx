import {
    Box,
    Button,
    Center,
    Drawer,
    DrawerBody,
    DrawerCloseButton,
    DrawerContent,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    Flex,
    Grid,
    GridItem,
    Image,
    Spacer,
    Text,
} from "@chakra-ui/react";
import React, { useRef } from "react";
import { useAsync } from "react-async";
import { Asset } from "./App";
import Bitcoin from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";
import { signAndSend, Trade, TradeSide } from "./wasmProxy";

interface ConfirmSwapDrawerProps {
    onCancel: () => void;
    onSwapped: (txId: string) => void;
    transaction: string;
    trade: Trade;
}

export default function ConfirmSwapDrawer(
    { onCancel, onSwapped, transaction, trade }: ConfirmSwapDrawerProps,
) {
    let { isPending, run } = useAsync({
        deferFn: async () => {
            let txId = await signAndSend(transaction);
            onSwapped(txId);
        },
    });

    const cancelButton = useRef(null);

    return <Drawer
        isOpen={true}
        placement="right"
        onClose={onCancel}
        initialFocusRef={cancelButton}
    >
        <form
            onSubmit={async e => {
                e.preventDefault();

                run();
            }}
            data-cy="confirm-swap-form"
        >
            <DrawerOverlay>
                <DrawerContent>
                    <DrawerCloseButton />
                    <DrawerHeader>Confirm Swap</DrawerHeader>
                    <DrawerBody>
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
                    </DrawerBody>

                    <DrawerFooter>
                        <Button
                            size="md"
                            mr={3}
                            onClick={onCancel}
                            ref={cancelButton}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            size="md"
                            variant="wallet_button"
                            isLoading={isPending}
                            data-cy="sign-and-send-button"
                        >
                            Sign and send
                        </Button>
                    </DrawerFooter>
                </DrawerContent>
            </DrawerOverlay>
        </form>
    </Drawer>;
}

interface YouSwapItemProps {
    tradeSide: TradeSide;
    action: "send" | "receive";
}

function YouSwapItem({
    tradeSide: {
        ticker,
        amount,
        balanceBefore,
        balanceAfter,
    },
    action,
}: YouSwapItemProps) {
    return (
        <Box w="100%">
            <Flex>
                <Box h="40px" p="1">
                    <Text>You {action}</Text>
                </Box>
                <Spacer />
                <Box w="40px" h="40px">
                    {ticker === Asset.LBTC && <Image src={Bitcoin} h="32px" />}
                    {ticker === Asset.USDT && <Image src={Usdt} h="32px" />}
                </Box>
                <Box h="40px" justify="right" p="1">
                    <Text align="center" justify="center">
                        {ticker}
                    </Text>
                </Box>
            </Flex>
            <Box bg="gray.100" borderRadius={"md"}>
                <Center>
                    <Box align="center" w="90%">
                        <Grid
                            templateRows="repeat(2, 2fr)"
                            templateColumns="repeat(2, 1fr)"
                        >
                            <GridItem colSpan={1}>
                                <Text align="left" textStyle="info">
                                    Old balance
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="info">
                                    {balanceBefore}
                                </Text>
                            </GridItem>
                            <GridItem colSpan={4}>
                                <Text align="center" textStyle="lg">
                                    {amount}
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="left" textStyle="info">
                                    New balance
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="info">
                                    {balanceAfter}
                                </Text>
                            </GridItem>
                        </Grid>
                    </Box>
                </Center>
            </Box>
        </Box>
    );
}
