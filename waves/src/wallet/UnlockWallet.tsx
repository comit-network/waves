import {
    Button,
    Drawer,
    DrawerBody,
    DrawerCloseButton,
    DrawerContent,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    Input,
    useDisclosure,
} from "@chakra-ui/react";
import React, { MouseEvent } from "react";

interface UnlockWalletProps {
    onUnlocked: (unlocked: boolean) => void;
}

function UnlockWallet({ onUnlocked }: UnlockWalletProps) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);

    const onUnlock = (_clicked: MouseEvent) => {
        // TODO implement wallet logic
        _clicked.preventDefault();
        onUnlocked(true);
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
                Unlock Wallet
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
                        <DrawerHeader>Unlock Wallet</DrawerHeader>
                        <DrawerBody>
                            <Input placeholder="Your top secret password" />
                        </DrawerBody>

                        <DrawerFooter>
                            <Button
                                size="md"
                                mr={3}
                                onClick={onClose}
                            >
                                Cancel
                            </Button>
                            <Button
                                size="md"
                                variant="wallet_button"
                                onClick={onUnlock}
                            >
                                Unlock
                            </Button>
                        </DrawerFooter>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}

export default UnlockWallet;
