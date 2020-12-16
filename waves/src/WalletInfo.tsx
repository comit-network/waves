import {
    Box,
    Button,
    Drawer,
    DrawerBody,
    DrawerCloseButton,
    DrawerContent,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    HStack,
    Image,
    Text,
    useDisclosure,
} from "@chakra-ui/react";
import React from "react";
import { WalletBalance } from "./App";
import Btc from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";

interface WalletInfoProps {
    balance: WalletBalance;
}

export default function WalletInfo({ balance }: WalletInfoProps) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);

    return (
        <>
            <Box as={Button} onClick={onOpen} ref={btnRef} bg="#FFFFFF">
                <HStack
                    align="left"
                >
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
                        </DrawerBody>

                        <DrawerFooter>
                            <Button
                                size="md"
                                variant="wallet_button"
                                onClick={onClose}
                            >
                                Close
                            </Button>
                        </DrawerFooter>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}
