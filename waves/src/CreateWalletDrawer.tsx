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
} from "@chakra-ui/react";
import React, { ChangeEvent, useRef } from "react";
import { useAsync } from "react-async";
import { newWallet } from "./wasmProxy";

interface CreateWalletDrawerProps {
    onCancel: () => void;
    onCreate: () => Promise<void>;
}

export default function CreateWalletDrawer({ onCancel, onCreate }: CreateWalletDrawerProps) {
    const [walletName] = React.useState("wallet-1");
    const [password, setPassword] = React.useState("");
    const onPasswordChange = (event: ChangeEvent<HTMLInputElement>) => setPassword(event.target.value);
    const passwordField = useRef(null);

    let { run, isPending } = useAsync({
        deferFn: async () => {
            await newWallet(password);
            await onCreate();
        },
    });

    return <Drawer
        isOpen={true}
        placement="right"
        onClose={onCancel}
        initialFocusRef={passwordField}
    >
        <form
            onSubmit={e => {
                e.preventDefault();
                run();
            }}
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
                            ref={passwordField}
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
                            Create
                        </Button>
                    </DrawerFooter>
                </DrawerContent>
            </DrawerOverlay>
        </form>
    </Drawer>;
}
