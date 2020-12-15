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
import React, { ChangeEvent, MouseEvent } from "react";
import { useHistory } from "react-router-dom";
import { newWallet } from "./wasmProxy";

function UnlockWallet() {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);
    const history = useHistory();
    const [walletName] = React.useState("wallet-1");
    const [password, setPassword] = React.useState("");
    const onPasswordChange = (event: ChangeEvent<HTMLInputElement>) => setPassword(event.target.value);

    const onCreate = async (_clicked: MouseEvent) => {
        const walletStatus = await newWallet(password);
        if (walletStatus.loaded) {
            _clicked.preventDefault();
            onClose();
            history.push("/swap");
        } else {
            console.log("Not unlocked. "); // TODO : show error
        }
    };

    return (
        <>
            <Button
                ref={btnRef}
                onClick={onOpen}
                size="lg"
                variant="main_button"
            >
                Create new wallet
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
                        <DrawerHeader>Create Wallet</DrawerHeader>
                        <DrawerBody>
                            <Input
                                pr="4.5rem"
                                type={"text"}
                                placeholder="Wallet name"
                                value={walletName}
                                readOnly
                            />
                            <Input
                                pr="4.5rem"
                                type={"password"}
                                placeholder="Wallet password"
                                value={password}
                                onChange={onPasswordChange}
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
                                onClick={onCreate}
                            >
                                Create
                            </Button>
                        </DrawerFooter>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}

export default UnlockWallet;
