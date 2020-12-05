import { Box, Button, Flex, HStack, Text, VStack } from "@chakra-ui/react";
import React, { MouseEvent } from "react";
import { RingLoader } from "react-spinners";
import "./App.css";
import AssetSelector from "./components/AssetSelector";
import SwapWithWallet from "./wallet/SwapWithWallet";
import UnlockWallet from "./wallet/UnlockWallet";

export enum AssetType {
    BTC = "BTC",
    USDT = "USDT",
}

function App() {
    const [alphaAsset, setAlphaAsset] = React.useState(AssetType.BTC);
    const [alphaAmount, setAlphaAmount] = React.useState(1);

    const [betaAsset, setBetaAsset] = React.useState(AssetType.USDT);
    const [betaAmount, setBetaAmount] = React.useState(191.13);

    const [walletUnlocked, setWalletUnlocked] = React.useState(false);
    const [publishedTx, setPublishedTx] = React.useState("");
    const [txPending, setTxPending] = React.useState(false);

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

    const onUnlocked = (unlocked: boolean) => {
        console.log(`Wallet unlocked ${unlocked}`);
        setWalletUnlocked(unlocked);
    };

    const isEmpty = (str: string) => {
        return (!str || 0 === str.length);
    };

    const onConfirmed = (txId: string) => {
        console.log(`Transaction published ${txId}`);
        setTxPending(true);
        setPublishedTx(txId);

        setTimeout(() => {
            setTxPending(false);
        }, 3000);
    };

    const openBlockExplorer = (_clicked: MouseEvent) => {
        window.open(`https://blockstream.info/liquid/tx/${publishedTx}`, "_blank");
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
                            {!walletUnlocked
                                && <UnlockWallet onUnlocked={onUnlocked} />}
                            {walletUnlocked && isEmpty(publishedTx)
                                && <SwapWithWallet
                                    onConfirmed={onConfirmed}
                                    alphaAmount={alphaAmount}
                                    betaAmount={betaAmount}
                                    alphaAsset={alphaAsset}
                                    betaAsset={betaAsset}
                                />}
                            {walletUnlocked && !isEmpty(publishedTx)
                                && <Button
                                    isLoading={txPending}
                                    size="lg"
                                    variant="main_button"
                                    spinner={<RingLoader size={50} color="white" />}
                                    onClick={openBlockExplorer}
                                >
                                    Check Transaction
                                </Button>}
                        </VStack>
                    </Box>
                </Flex>
            </header>
        </div>
    );
}

export default App;
