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
import useSWR from "swr";
import { AssetType } from "./App";
import Bitcoin from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";
import { decomposeTransaction, signAndSend } from "./wasmProxy";

interface ConfirmSwapDrawerProps {
    isOpen: boolean;
    onCancel: () => void;
    onSwapped: (txId: string) => void;
    transaction: string;
}

export default function ConfirmSwapDrawer({ isOpen, onCancel, onSwapped, transaction }: ConfirmSwapDrawerProps) {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    let { data, error } = useSWR("decompose-transaction", () => decomposeTransaction(transaction));
    let { isPending, run } = useAsync({
        deferFn: async () => {
            let txId = await signAndSend(transaction);
            onSwapped(txId);
        },
    });
    const cancelButton = useRef(null);

    // TODO: get these out of data
    let alphaAsset = AssetType.BTC;
    let alphaAmount = 0;
    let betaAsset = AssetType.USDT;
    let betaAmount = 0;

    return <Drawer
        isOpen={isOpen}
        placement="right"
        onClose={onCancel}
        initialFocusRef={cancelButton}
    >
        <form
            onSubmit={async e => {
                e.preventDefault();

                run();
            }}
        >
            <DrawerOverlay>
                <DrawerContent>
                    <DrawerCloseButton />
                    <DrawerHeader>Confirm Swap</DrawerHeader>
                    <DrawerBody>
                        <Box>
                            <YouSwapItem
                                asset={alphaAsset}
                                amount={alphaAmount}
                                balanceAfter={0}
                                balanceBefore={0}
                            />
                        </Box>
                        <Box>
                            <YouSwapItem
                                asset={betaAsset}
                                amount={betaAmount}
                                balanceAfter={0}
                                balanceBefore={0}
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
    asset: AssetType;
    amount: number;
    balanceBefore: number;
    balanceAfter: number;
}

function YouSwapItem({
    asset,
    amount,
    balanceBefore,
    balanceAfter,
}: YouSwapItemProps) {
    return (
        <Box w="100%">
            <Flex>
                <Box h="40px" p="1">
                    <Text>You send</Text>
                </Box>
                <Spacer />
                <Box w="40px" h="40px">
                    {asset === AssetType.BTC && <Image src={Bitcoin} h="32px" />}
                    {asset === AssetType.USDT && <Image src={Usdt} h="32px" />}
                </Box>
                <Box h="40px" justify="right" p="1">
                    <Text align="center" justify="center">
                        {asset}
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
