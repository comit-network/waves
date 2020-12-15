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
import React, { ChangeEvent, Dispatch, MouseEvent } from "react";
import { useHistory } from "react-router-dom";
import { Action } from "./App";
import { unlockWallet } from "./wasmProxy";

interface UnlockWalletProps {
    dispatch: Dispatch<Action>;
}
function UnlockWallet({ dispatch }: UnlockWalletProps) {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const btnRef = React.useRef(null);
    const history = useHistory();
    const [password, setPassword] = React.useState("");
    const onPasswordChange = (event: ChangeEvent<HTMLInputElement>) => setPassword(event.target.value);

    const onUnlock = async (_clicked: MouseEvent) => {
        const walletStatus = await unlockWallet(password);
        if (walletStatus.loaded) {
            _clicked.preventDefault();
            dispatch({
                type: "UpdateWallet",
                value: {
                    exists: true,
                    loaded: true,
                    btcBalance: 0,
                    usdtBalance: 0,
                },
            });
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
