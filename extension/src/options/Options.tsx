import {
    Box,
    Button,
    Center,
    FormControl,
    FormErrorMessage,
    FormHelperText,
    FormLabel,
    HStack,
    Input,
    InputGroup,
    InputRightElement,
    Radio,
    RadioGroup,
    VStack,
} from "@chakra-ui/react";
import Debug from "debug";
import { ChangeEvent, useState } from "react";
import * as React from "react";
import { loadLoanBackup } from "../background-proxy";
import "./Options.css";

Debug.enable("*");
const debug = Debug("options");

function Options() {
    const storedChain = localStorage.getItem("CHAIN") ? localStorage.getItem("CHAIN")!.toLowerCase() : "UNDEFINED";
    const [writtenChain, writeChain] = useState(storedChain);

    const saveChainState = (value: string) => {
        localStorage.setItem("CHAIN", value);
        writeChain(value);
    };

    const [error, setError] = useState("");
    const [isError, setIsError] = useState(false);

    const handleUpload = (e: ChangeEvent<HTMLInputElement>) => {
        if (!e.target || !e.target.files) {
            setError("No file selected");
            setIsError(true);
            return;
        }

        const fileReader = new FileReader();
        let file = e.target.files[0];
        fileReader.readAsText(file, "UTF-8");
        fileReader.onload = async (e) => {
            let backupDetails = JSON.parse(e.target!.result as string);
            await loadLoanBackup(backupDetails);
        };
    };

    return (
        <Center>
            <VStack>
                <Box>
                    <FormControl as="fieldset" isRequired>
                        <HStack>
                            <FormLabel as="legend">Chain</FormLabel>
                            <RadioGroup defaultValue={storedChain} value={writtenChain} onChange={saveChainState}>
                                <HStack spacing="24px">
                                    <Radio value="elements">Elements</Radio>
                                    <Radio value="liquid">Liquid</Radio>
                                </HStack>
                            </RadioGroup>
                        </HStack>
                    </FormControl>

                    <KeyValueField keyName="ESPLORA_API_URL" title={"Esplora API URL"} />
                    <KeyValueField keyName="LBTC_ASSET_ID" title={"Bitcoin Asset ID (L-BTC)"} />
                    <KeyValueField keyName="LUSDT_ASSET_ID" title={"USD Asset ID (L-USDT)"} />
                </Box>
                <Box>
                    <FormControl id="backup" isInvalid={isError}>
                        <FormLabel>Restore backup</FormLabel>
                        <input type="file" onChange={handleUpload} />
                        <FormHelperText>Upload a loan-details.json file</FormHelperText>
                        <FormErrorMessage>{error}</FormErrorMessage>
                    </FormControl>
                </Box>
            </VStack>
        </Center>
    );
}

interface KeyValueFieldProps {
    keyName: string;
    title: string;
}

function KeyValueField({ keyName, title }: KeyValueFieldProps) {
    const storedValue = localStorage.getItem(keyName)
        ? localStorage.getItem(keyName)!.toLowerCase()
        : "UNDEFINED";
    const [writtenValue, writeValue] = useState(storedValue);

    return (
        <form
            onSubmit={(e) => {
                e.preventDefault();
                debug(`Saving ${writtenValue}`);
                localStorage.setItem(keyName, writtenValue);
            }}
        >
            <FormControl as="fieldset" isRequired>
                <HStack>
                    <InputGroup>
                        <FormLabel as="legend">{title}</FormLabel>
                        <Input
                            type={"text"}
                            value={writtenValue}
                            onChange={(e) => writeValue(e.target.value)}
                        >
                        </Input>
                        <InputRightElement>
                            <Button
                                type={"submit"}
                            >
                                Save
                            </Button>
                        </InputRightElement>
                    </InputGroup>
                </HStack>
            </FormControl>
        </form>
    );
}

export default Options;
