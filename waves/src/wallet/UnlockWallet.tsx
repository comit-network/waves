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
import { Link as RouterLink, useHistory } from "react-router-dom";

function UnlockWallet() {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);
    const history = useHistory();

    const onUnlock = (_clicked: MouseEvent) => {
        // TODO implement wallet logic
        _clicked.preventDefault();
        history.push("/swap");
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
                            <Input
                                pr="4.5rem"
                                type={"password"}
                                placeholder="Enter password"
                            />
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
