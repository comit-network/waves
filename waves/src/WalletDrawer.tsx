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
    FormControl,
    FormErrorMessage,
    HStack,
    Image,
    Input,
    Text,
    VStack,
} from "@chakra-ui/react";
import Debug from "debug";
import QRCode from "qrcode.react";
import React, { ChangeEvent } from "react";
import { Async, useAsync } from "react-async";
import { Balances } from "./App";
import { fundAddress } from "./Bobtimus";
import Btc from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";
import { getAddress, withdrawAll } from "./wasmProxy";

const debug = Debug("wallet");

export interface WalletDrawerProps {
    onClose: () => void;
    balances: Balances;
    reloadBalances: () => Promise<void>;
}

export default function WalletDrawer({ onClose, balances, reloadBalances }: WalletDrawerProps) {
    const [withdrawAddress, setWithdrawAddress] = React.useState("");
    const handleWithdrawAddress = (event: ChangeEvent<HTMLInputElement>) => setWithdrawAddress(event.target.value);

    let { isLoading: isWithdrawing, isRejected: withdrawFailed, run: withdraw } = useAsync({
        deferFn: ([address]) => withdrawAll(address),
        onReject: (error) => debug("failed to withdraw funds: %s", error),
    });

    let { isLoading: isFunding, run: fund } = useAsync({
        deferFn: async () => {
            let address = await getAddress();
            await fundAddress(address);
            await reloadBalances();
        },
        onReject: (error) => debug("failed to fund wallet: %s", error),
    });

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
                            <form
                                onSubmit={e => {
                                    e.preventDefault();
                                    withdraw(withdrawAddress);
                                }}
                            >
                                <Text textStyle="actionable">Withdraw:</Text>
                                <HStack>
                                    <FormControl id="password" isInvalid={withdrawFailed}>
                                        <Input
                                            placeholder="Address"
                                            size="md"
                                            bg={"white"}
                                            value={withdrawAddress}
                                            onChange={handleWithdrawAddress}
                                        />
                                        <FormErrorMessage>Failed to withdraw funds.</FormErrorMessage>
                                    </FormControl>
                                    <Button
                                        type="submit"
                                        variant="primary"
                                        isLoading={isWithdrawing}
                                    >
                                        Send
                                    </Button>
                                </HStack>
                            </form>
                        </VStack>
                        {process.env.NODE_ENV === "development" && (
                            <VStack
                                bg="gray.100"
                                align="center"
                                borderRadius={"md"}
                                p={1}
                            >
                                <Button
                                    variant="primary"
                                    onClick={fund}
                                    isLoading={isFunding}
                                >
                                    Fund
                                </Button>
                            </VStack>
                        )}
                    </VStack>
                </DrawerBody>

                <DrawerFooter>
                    <Button variant="primary" onClick={onClose}>
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
                        <Text textStyle="lgGray">Address</Text>
                        <QRCode value={data} size={100} />
                        <Text textStyle="mdGray" maxWidth={"15em"} isTruncated data-cy="wallet-address-textfield">
                            {data}
                        </Text>
                    </VStack>
                );
            }
            return null;
        }}
    </Async>
);
