import {
    Button,
    Drawer,
    DrawerBody,
    DrawerCloseButton,
    DrawerContent,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    Text,
    useDisclosure,
} from "@chakra-ui/react";
import React, { MouseEvent } from "react";
import { AssetType } from "../App";

interface SwapWithWalletProps {
    alphaAmount: number;
    alphaAsset: AssetType;
    betaAmount: number;
    betaAsset: AssetType;
    onConfirmed: (txId: string) => void;
}

const DEFAULT_TX_ID = "7565865560cdef747c5358ca9ff46747a82617292452b6392d0d77072701c413";

function SwapWithWallet({ alphaAmount, alphaAsset, betaAmount, betaAsset, onConfirmed }: SwapWithWalletProps) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);

    const onConfirm = (_clicked: MouseEvent) => {
        // TODO implement wallet logic
        _clicked.preventDefault();
        onConfirmed(DEFAULT_TX_ID);
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
                Confirm Swap
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
                        <DrawerHeader>Confirm Your Swap</DrawerHeader>
                        <DrawerBody>
                            <Text textStyle="info">You send {alphaAmount} {alphaAsset}</Text>
                            <Text textStyle="info">You receive {betaAmount} {betaAsset}</Text>
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
