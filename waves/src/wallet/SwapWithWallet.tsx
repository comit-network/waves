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
    useDisclosure,
} from "@chakra-ui/react";
import React, { Dispatch, MouseEvent } from "react";
import { useHistory } from "react-router-dom";
import { Action, AssetType } from "../App";
import Bitcoin from "../components/bitcoin.svg";
import Usdt from "../components/tether.svg";

interface SwapWithWalletProps {
    alphaAmount: number;
    alphaAsset: AssetType;
    betaAmount: number;
    betaAsset: AssetType;
    onConfirmed: (txId: string) => void;
    dispatch: Dispatch<Action>;
}

const DEFAULT_TX_ID = "7565865560cdef747c5358ca9ff46747a82617292452b6392d0d77072701c413";

function SwapWithWallet(
    { alphaAmount, alphaAsset, betaAmount, betaAsset, onConfirmed, dispatch }: SwapWithWalletProps,
) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);
    const history = useHistory();

    const onConfirm = (_clicked: MouseEvent) => {
        // TODO implement wallet logic
        _clicked.preventDefault();
        onConfirmed(DEFAULT_TX_ID);
        dispatch({
            type: "PublishTransaction",
            value: DEFAULT_TX_ID,
        });
        history.push("/done");
        onClose();
    };

    return (
        <>
            <Button
                ref={btnRef}
                onClick={onOpen}
                size="lg"
                variant="main_button"
            >
                Swap
            </Button>
            <Drawer
                isOpen={isOpen}
                placement="right"
                onClose={onClose}
                finalFocusRef={btnRef}
            >
                <DrawerOverlay>
                    <DrawerContent>
                        <DrawerCloseButton />
                        <DrawerHeader>Confirm Swap</DrawerHeader>
                        <DrawerBody>
                            {/*<VStack>*/}
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
                            {/*</VStack>*/}
                        </DrawerBody>

                        <DrawerFooter>
                            <Button size="md" mr={3} onClick={onClose}>
                                Cancel
                            </Button>
                            <Button size="md" variant="wallet_button" onClick={onConfirm}>Sign and Send</Button>
                        </DrawerFooter>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}

export default SwapWithWallet;

interface YouSwapItemProps {
    asset: AssetType;
    amount: number;
    balanceBefore: number;
    balanceAfter: number;
}

function YouSwapItem({ asset, amount, balanceBefore, balanceAfter }: YouSwapItemProps) {
    return (
        <Box w="100%">
            <Flex>
                <Box h="40px" p="1">
                    <Text>You send</Text>
                </Box>
                <Spacer />
                <Box w="40px" h="40px">
                    {asset === AssetType.BTC
                        && <Image src={Bitcoin} h="32px" />}
                    {asset === AssetType.USDT
                        && <Image src={Usdt} h="32px" />}
                </Box>
                <Box h="40px" justify="right" p="1">
                    <Text align="center" justify="center">{asset}</Text>
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
                                <Text align="left" textStyle="info">Old balance</Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="info">{balanceBefore}</Text>
                            </GridItem>
                            <GridItem colSpan={4}>
                                <Text align="center" textStyle="lg">{amount}</Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="left" textStyle="info">New balance</Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="info">{balanceAfter}</Text>
                            </GridItem>
                        </Grid>
                    </Box>
                </Center>
            </Box>
        </Box>
    );
}
