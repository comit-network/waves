import {
    Button,
    Drawer,
    DrawerBody,
    DrawerCloseButton,
    DrawerContent,
    DrawerFooter,
    DrawerHeader,
    DrawerOverlay,
    FormControl,
    FormErrorMessage,
    FormLabel,
    Input,
} from "@chakra-ui/react";
import Debug from "debug";
import React, { ChangeEvent, useRef, useState } from "react";
import { useAsync } from "react-async";
import { unlockWallet } from "./wasmProxy";

const debug = Debug("wallet");

interface UnlockWalletDrawerProps {
    onCancel: () => void;
    onUnlock: () => Promise<void>;
}

export default function UnlockWalletDrawer({ onCancel, onUnlock }: UnlockWalletDrawerProps) {
    const [password, setPassword] = useState("");
    const onPasswordChange = (event: ChangeEvent<HTMLInputElement>) => setPassword(event.target.value);
    const passwordField = useRef(null);

    let { run, isPending, isRejected } = useAsync({
        deferFn: async () => {
            await unlockWallet(password);
            await onUnlock();
        },
        onReject: (e) => debug("failed to unlock wallet: %s", e),
    });

    return <Drawer
        isOpen={true}
        placement="right"
        onClose={onCancel}
        initialFocusRef={passwordField}
    >
        <DrawerOverlay>
            <DrawerContent>
                <form
                    onSubmit={async e => {
                        e.preventDefault();
                        run();
                    }}
                >
                    <DrawerCloseButton />
                    <DrawerHeader>Unlock Wallet</DrawerHeader>
                    <DrawerBody>
                        <FormControl id="password" isInvalid={isRejected}>
                            <FormLabel>Password</FormLabel>
                            <Input
                                ref={passwordField}
                                pr="4.5rem"
                                type={"password"}
                                value={password}
                                onChange={onPasswordChange}
                            />
                            <FormErrorMessage>Failed to unlock wallet. Wrong password?</FormErrorMessage>
                        </FormControl>
                    </DrawerBody>

                    <DrawerFooter>
                        <Button
                            size="md"
                            mr={3}
                            onClick={onCancel}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            size="md"
                            variant="wallet_button"
                            isLoading={isPending}
                        >
                            Unlock
                        </Button>
                    </DrawerFooter>
                </form>
            </DrawerContent>
        </DrawerOverlay>
    </Drawer>;
}
