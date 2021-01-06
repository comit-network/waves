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
    HStack,
    Image,
    Input,
    Text,
    VStack,
} from "@chakra-ui/react";
import Debug from "debug";
import QRCode from "qrcode.react";
import React, { ChangeEvent } from "react";
import { Async } from "react-async";
import { Balances } from "./App";
import { fundAddress } from "./Bobtimus";
import Btc from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";
import { getAddress, withdrawAll } from "./wasmProxy";

const debug = Debug("wallet");

export interface WalletDrawerProps {
    onClose: () => void;
    balances: Balances;
}

export default function WalletDrawer({ onClose, balances }: WalletDrawerProps) {
    const [withdrawAddress, setWithdrawAddress] = React.useState("");
    const handleWithdrawAddress = (event: ChangeEvent<HTMLInputElement>) => setWithdrawAddress(event.target.value);

    const withdraw = async () => {
        let txId = await withdrawAll(withdrawAddress);
        debug("Withdrew everything. Resulting txId: %s", txId);
    };

    async function fundWallet(): Promise<any> {
        let address = await getAddress();
        await fundAddress(address);
    }

    return <Drawer
        isOpen={true}
        placement="right"
        onClose={onClose}
    >
        <DrawerOverlay>
            <DrawerContent>
                <DrawerCloseButton />
                <DrawerHeader>Wallet</DrawerHeader>
                <DrawerBody>
                    <VStack align="stretch" spacing={4}>
                        <Center
                            bg="gray.100"
                            h="10em"
                            color="white"
                            borderRadius={"md"}
                        >
                            <AddressQr />
                        </Center>
                        <VStack bg="gray.100" align="left" borderRadius={"md"} p={1}>
                            <Box>
                                <HStack>
                                    <Box>
                                        <Image src={Usdt} h="20px" />
                                    </Box>
                                    <Box>
                                        <Text>L-USDT: {balances.usdt}</Text>
                                    </Box>
                                </HStack>
                            </Box>
                            <Box>
                                <HStack>
                                    <Box>
                                        <Image src={Btc} h="20px" />
                                    </Box>
                                    <Box>
                                        <Text>L-BTC: {balances.btc}</Text>
                                    </Box>
                                </HStack>
                            </Box>
                        </VStack>
                        <VStack bg="gray.100" align="center" borderRadius={"md"} p={1}>
                            <Text textStyle="actionable">Withdraw everything to:</Text>
                            <HStack>
                                <Input
                                    placeholder="Address"
                                    size="md"
                                    bg={"white"}
                                    value={withdrawAddress}
                                    onChange={handleWithdrawAddress}
                                />
                                <Button
                                    size="md"
                                    variant="wallet_button"
                                    onClick={withdraw}
                                >
                                    Send
                                </Button>
                            </HStack>
                        </VStack>
                        {process.env.NODE_ENV === "development" && (
                            <VStack
                                bg="gray.100"
                                align="center"
                                borderRadius={"md"}
                                p={1}
                            >
                                <Button
                                    size="md"
                                    variant="wallet_button"
                                    onClick={fundWallet}
                                >
                                    Fund
                                </Button>
                            </VStack>
                        )}
                    </VStack>
                </DrawerBody>

                <DrawerFooter>
                    <Button size="md" variant="wallet_button" onClick={onClose}>
                        Close
                    </Button>
                </DrawerFooter>
            </DrawerContent>
        </DrawerOverlay>
    </Drawer>;
}

const AddressQr = () => (
    <Async promiseFn={getAddress}>
        {({ data, error, isPending }) => {
            if (isPending) return "Loading...";
            if (error) return `Something went wrong: ${error.message}`;
            if (data) {
                return (
                    <VStack>
                        <Text textStyle="actionable">Address</Text>
                        <QRCode value={data} size={100} />
                        <Text textStyle="addressInfo" maxWidth={"15em"} isTruncated>
                            {data}
                        </Text>
                    </VStack>
                );
            }
            return null;
        }}
    </Async>
);
