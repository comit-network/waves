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
    useDisclosure,
    VStack,
} from "@chakra-ui/react";
import QRCode from "qrcode.react";
import React, { ChangeEvent } from "react";
import { Async } from "react-async";
import { WalletBalance } from "./App";
import Btc from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";
import { getAddress, withdrawAll } from "./wasmProxy";

interface WalletInfoProps {
    balance: WalletBalance;
}

export default function WalletInfo({ balance }: WalletInfoProps) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);
    const [withdrawAddress, setWithdrawAddress] = React.useState("");

    const handleWithdrawAddress = (event: ChangeEvent<HTMLInputElement>) => setWithdrawAddress(event.target.value);

    const withdraw = async () => {
        let txId = await withdrawAll(withdrawAddress);
        console.log("Withdrew everything. Resulting txId: {}", txId);
    };

    async function fundWallet(): Promise<any> {
        let address = await getAddress();
        await fetch("/faucet/" + address, {
            method: "POST",
        });
    }

    return (
        <>
            <Box as={Button} onClick={onOpen} ref={btnRef} bg="#FFFFFF">
                <HStack align="left">
                    <Box>
                        <HStack>
                            <Box>
                                <Image src={Usdt} h="20px" />
                            </Box>
                            <Box>
                                <Text textStyle="info">L-USDT: {balance.usdtBalance}</Text>
                            </Box>
                        </HStack>
                    </Box>
                    <Box>
                        <HStack>
                            <Box>
                                <Image src={Btc} h="20px" />
                            </Box>
                            <Box>
                                <Text textStyle="info">L-BTC: {balance.btcBalance}</Text>
                            </Box>
                        </HStack>
                    </Box>
                </HStack>
            </Box>
            <Drawer
                isOpen={isOpen}
                placement="right"
                onClose={onClose}
                finalFocusRef={btnRef}
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
                                                <Text>L-USDT: {balance.usdtBalance}</Text>
                                            </Box>
                                        </HStack>
                                    </Box>
                                    <Box>
                                        <HStack>
                                            <Box>
                                                <Image src={Btc} h="20px" />
                                            </Box>
                                            <Box>
                                                <Text>L-BTC: {balance.btcBalance}</Text>
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
            </Drawer>
        </>
    );
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
