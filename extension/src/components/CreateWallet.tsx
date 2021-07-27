import { RepeatIcon } from "@chakra-ui/icons";
import {
    Accordion,
    AccordionButton,
    AccordionIcon,
    AccordionItem,
    AccordionPanel,
    Box,
    Button,
    Checkbox,
    FormControl,
    FormErrorMessage,
    IconButton,
    Input,
    InputGroup,
    InputRightElement,
    Textarea,
} from "@chakra-ui/react";
import Debug from "debug";
import * as React from "react";
import { ChangeEvent, useState } from "react";
import { useAsync } from "react-async";
import { bip39SeedWords, createWalletFromBip39 } from "../background-proxy";

Debug.enable("*");
const debug = Debug("unlock-wallet");

type CreateWalletProps = {
    onUnlock: () => void;
};

function CreateWallet({ onUnlock }: CreateWalletProps) {
    const [backedUp, setBackedUp] = useState(false);
    const [seedWords, setSeedWords] = useState(
        "chase prevent symptom panel promote short tray cigar wonder vanish sustain hurry",
    );
    const [backedUpSeedWords, setBackedUpSeedWords] = useState("");
    const [show, setShow] = useState(false);
    const [password, setPassword] = useState("");

    const onPasswordChange = (event: ChangeEvent<HTMLInputElement>) => setPassword(event.target.value);
    const toggleShowPassword = () => setShow(!show);

    let { run: createWallet, isPending: isCreatingWallet, isRejected: createWalletIsRejected } = useAsync({
        deferFn: async () => {
            await createWalletFromBip39(backedUpSeedWords, password);
            onUnlock();
        },
        onReject: (e) => debug("Failed to unlock wallet: %s", e),
    });
    let { run: newSeedWords, isPending: isGeneratingSeedWords, isRejected: generatingSeedWordsFailed } = useAsync({
        deferFn: async () => {
            let words = await bip39SeedWords();
            setSeedWords(words);
        },
        onReject: (e) => debug("Failed to unlock wallet: %s", e),
    });

    return (
        <Accordion>
            <AccordionItem>
                <h2>
                    <AccordionButton>
                        <Box flex="1" textAlign="left">
                            Generate seed words
                        </Box>
                        <AccordionIcon />
                    </AccordionButton>
                </h2>
                <AccordionPanel pb={4}>
                    <Textarea
                        placeholder={seedWords}
                        value={seedWords}
                        isInvalid={generatingSeedWordsFailed}
                        onChange={(event) => setSeedWords(event.target.value)}
                    />
                    <IconButton
                        aria-label="Refresh"
                        icon={<RepeatIcon />}
                        isLoading={isGeneratingSeedWords}
                        onClick={(_) => {
                            newSeedWords();
                        }}
                    />
                    <Checkbox isChecked={backedUp} onChange={_ => setBackedUp(!backedUp)}>
                        I confirm that I have a secure backup of the seed words
                    </Checkbox>
                </AccordionPanel>
            </AccordionItem>

            <AccordionItem isDisabled={!backedUp}>
                <h2>
                    <AccordionButton>
                        <Box flex="1" textAlign="left">
                            Confirm seed words
                        </Box>

                        <AccordionIcon />
                    </AccordionButton>
                </h2>
                <AccordionPanel pb={4}>
                    <form
                        onSubmit={async e => {
                            e.preventDefault();
                            await createWallet();
                        }}
                    >
                        <Textarea
                            placeholder={"Your seed words..."}
                            value={backedUpSeedWords}
                            onChange={(event) => setBackedUpSeedWords(event.target.value)}
                        />
                        <FormControl id="password" isInvalid={createWalletIsRejected}>
                            <InputGroup size="md">
                                <Input
                                    pr="4.5rem"
                                    type={show ? "text" : "password"}
                                    placeholder="Enter password"
                                    value={password}
                                    onChange={onPasswordChange}
                                    data-cy={"data-cy-create-wallet-password-input"}
                                />
                                <InputRightElement width="4.5rem">
                                    <Button
                                        h="1.75rem"
                                        size="sm"
                                        onClick={toggleShowPassword}
                                    >
                                        {show ? "Hide" : "Show"}
                                    </Button>
                                </InputRightElement>
                            </InputGroup>
                            <FormErrorMessage>Failed to unlock wallet. Wrong password?</FormErrorMessage>
                        </FormControl>
                        <Button
                            type="submit"
                            variant="solid"
                            isLoading={isCreatingWallet}
                            data-cy={"data-cy-create-wallet-button"}
                        >
                            {"Create"}
                        </Button>
                    </form>
                </AccordionPanel>
            </AccordionItem>
        </Accordion>
    );
}

export default CreateWallet;
