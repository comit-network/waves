import { ChevronDownIcon } from "@chakra-ui/icons";
import {
    Box,
    Button,
    Drawer,
    DrawerBody,
    DrawerContent,
    DrawerHeader,
    DrawerOverlay,
    Grid,
    HStack,
    Image,
    Text,
    useDisclosure,
    VStack,
} from "@chakra-ui/react";
import React from "react";
import { AssetType } from "../App";
import Bitcoin from "./bitcoin.svg";
import Usdt from "./tether.svg";

interface CurrencySelectProps {
    type: AssetType;
    onAssetChange: (asset: AssetType) => void;
    placement: "left" | "right";
}

// TODO: make the select option nice as in the mock, i.e. with ticker symbols
function AssetSelect({ type, onAssetChange, placement }: CurrencySelectProps) {
    const onChange = (value: AssetType) => {
        onAssetChange(value);
        onClose();
    };
    const btnRef = React.useRef(null);

    const { isOpen, onOpen, onClose } = useDisclosure();

    return (
        <>
            <Button ref={btnRef} w="100%" bg="white" border="grey" size="lg" shadow="md" onClick={onOpen}>
                {type === "BTC" && <BitcoinSelect renderAsDropDown={true} />}
                {type === "USDT" && <UsdtSelect renderAsDropDown={true} />}
            </Button>

            <Drawer placement={placement} onClose={onClose} isOpen={isOpen} size="sm">
                <DrawerOverlay>
                    <DrawerContent>
                        <DrawerHeader>{"Available Trading Pairs"}</DrawerHeader>
                        <DrawerBody>
                            <Grid templateColumns="repeat(2, 1fr)" gap={6}>
                                <BitcoinBox onSelect={onChange} />
                                <UsdtBox onSelect={onChange} />
                            </Grid>
                        </DrawerBody>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}

export default AssetSelect;

interface SelectProps {
    renderAsDropDown: boolean;
}

function BitcoinSelect({ renderAsDropDown }: SelectProps) {
    return (
        <HStack spacing="24px">
            <Box h="40px">
                <Image src={Bitcoin} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-BTC : Bitcoin</Text>
            </Box>
            {renderAsDropDown && <Box>
                <ChevronDownIcon h="60%" color="gray.500" />
            </Box>}
        </HStack>
    );
}

function UsdtSelect({ renderAsDropDown }: SelectProps) {
    return (
        <HStack spacing="24px">
            <Box h="40px">
                <Image src={Usdt} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-USDT : Tether</Text>
            </Box>
            {renderAsDropDown && <Box>
                <ChevronDownIcon h="60%" color="gray.500" />
            </Box>}
        </HStack>
    );
}

interface CurrencyBoxProps {
    onSelect: (asset: AssetType) => void;
}

function BitcoinBox({ onSelect }: CurrencyBoxProps) {
    return (<Box h="100px" as={Button} onClick={(_e) => onSelect(AssetType.BTC)}>
        <VStack spacing="24px">
            <Box h="40px">
                <Image src={Bitcoin} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-BTC - Bitcoin</Text>
            </Box>
        </VStack>
    </Box>);
}
function UsdtBox({ onSelect }: CurrencyBoxProps) {
    return (<Box h="100px" as={Button} onClick={(e) => onSelect(AssetType.USDT)}>
        <VStack spacing="24px">
            <Box h="40px">
                <Image src={Usdt} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-USDT - Tether</Text>
            </Box>
        </VStack>
    </Box>);
}
