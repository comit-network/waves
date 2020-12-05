import { Box, Button, Flex, HStack, Text, VStack } from "@chakra-ui/react";
import React from "react";
import "./App.css";
import AssetSelector from "./components/AssetSelector";

export enum AssetType {
    BTC = "BTC",
    USDT = "USDT",
}

function App() {
    const [alphaAsset, setAlphaAsset] = React.useState(AssetType.BTC);
    const [alphaAmount, setAlphaAmount] = React.useState(1);

    const [betaAsset, setBetaAsset] = React.useState(AssetType.USDT);
    const [betaAmount, setBetaAmount] = React.useState(191.13);

    const onUpdateAlphaAssetType = (newType: AssetType) => {
        console.log(`Received new alpha assetType: ${newType}`);
        setAlphaAsset(newType);
    };
    const onUpdateBetaAssetType = (newType: AssetType) => {
        console.log(`Received new beta assetType: ${newType}`);
        setBetaAsset(newType);
    };

    const onUpdateAlphaAssetAmount = (newAmount: number) => {
        console.log(`Received new alpha amount: ${newAmount}`);
        setAlphaAmount(newAmount);
    };

    const onUpdateBetaAssetAmount = (newAmount: number) => {
        console.log(`Received new beta amount: ${newAmount}`);
        setBetaAmount(newAmount);
    };

    return (
        <div className="App">
            <header className="App-header">
                <Flex color="white">
                    <Box>
                        <VStack spacing={4}>
                            <HStack spacing={4}>
                                <AssetSelector
                                    amount={alphaAmount}
                                    type={alphaAsset}
                                    onTypeChange={onUpdateAlphaAssetType}
                                    onAmountChange={onUpdateAlphaAssetAmount}
                                />
                                <AssetSelector
                                    amount={betaAmount}
                                    type={betaAsset}
                                    onTypeChange={onUpdateBetaAssetType}
                                    onAmountChange={onUpdateBetaAssetAmount}
                                />
                            </HStack>
                            <Text textStyle="info">1 BTC = 19,337.42 USDT</Text>
                            <Button
                                size="lg"
                                bg="#304FFE"
                                rounded="md"
                                _hover={{ bg: "blue.300" }}
                            >
                                Unlock Wallet
                            </Button>
                        </VStack>
                    </Box>
                </Flex>
            </header>
        </div>
    );
}

export default App;
