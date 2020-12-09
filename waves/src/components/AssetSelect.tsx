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
    Modal,
    ModalBody,
    ModalCloseButton,
    ModalContent,
    ModalFooter,
    ModalHeader,
    ModalOverlay,
    Text,
    useDisclosure,
    VStack,
} from "@chakra-ui/react";
import React, { MouseEvent } from "react";
import { GrGithub } from "react-icons/gr";
import { AssetType } from "../App";
import Bitcoin from "./bitcoin.svg";
import Xmr from "./monero.svg";
import Usdt from "./tether.svg";

interface CurrencySelectProps {
    type: AssetType;
    onAssetChange: (asset: AssetType) => void;
    placement: "left" | "right";
}

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
                {type === "BTC" && <BitcoinSelect />}
                {type === "USDT" && <UsdtSelect />}
            </Button>

            <Drawer placement={placement} onClose={onClose} isOpen={isOpen} size="sm">
                <DrawerOverlay>
                    <DrawerContent>
                        <DrawerHeader>{"Available Assets"}</DrawerHeader>
                        <DrawerBody>
                            <Grid templateColumns="repeat(2, 1fr)" gap={6}>
                                <BitcoinBox onSelect={onChange} />
                                <UsdtBox onSelect={onChange} />
                                <XmrBox />
                            </Grid>
                        </DrawerBody>
                    </DrawerContent>
                </DrawerOverlay>
            </Drawer>
        </>
    );
}

export default AssetSelect;

function BitcoinSelect() {
    return (
        <HStack spacing="24px">
            <Box h="40px">
                <Image src={Bitcoin} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-BTC - Bitcoin</Text>
            </Box>
            <Box>
                <ChevronDownIcon h="60%" color="gray.500" />
            </Box>
        </HStack>
    );
}

function UsdtSelect() {
    return (
        <HStack spacing="24px">
            <Box h="40px">
                <Image src={Usdt} h="100%" />
            </Box>
            <Box>
                <Text textStyle="assetSelect">L-USDT - Tether</Text>
            </Box>
            <Box>
                <ChevronDownIcon h="60%" color="gray.500" />
            </Box>
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

function XmrBox() {
    const { isOpen, onOpen, onClose } = useDisclosure();
    const openXmrProject = (_clicked: MouseEvent) => {
        window.open(`https://github.com/comit-network/xmr-btc-swap/`, "_blank");
    };

    return (
        <>
            <Box h="100px" as={Button} onClick={onOpen}>
                <VStack spacing="24px">
                    <Box h="40px">
                        <Image src={Xmr} h="100%" />
                    </Box>
                    <Box>
                        <Text textStyle="assetSelect">XMR - Monero</Text>
                    </Box>
                </VStack>
            </Box>

            <Modal isOpen={isOpen} onClose={onClose}>
                <ModalOverlay />
                <ModalContent>
                    <ModalHeader>Unsupported</ModalHeader>
                    <ModalCloseButton />
                    <ModalBody>
                        <Text textStyle="lg">
                            Swapping BTC/XMR is currently not supported in the browser. Click below to checkout our
                            other project.
                        </Text>
                    </ModalBody>

                    <ModalFooter>
                        <Button leftIcon={<GrGithub />} size="md" variant="wallet_button" onClick={openXmrProject}>
                            Checkout on Github
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>
        </>
    );
}
