import { Center, NumberInput, NumberInputField, VStack } from "@chakra-ui/react";
import React from "react";
import { AssetType } from "../App";
import AssetSelect from "./AssetSelect";

interface AssetSelectorProps {
    type: AssetType;
    amount: number;
    onTypeChange: (asset: AssetType) => void;
    onAmountChange: (asset: number) => void;
}

function AssetSelector({ type, amount, onTypeChange, onAmountChange }: AssetSelectorProps) {
    const box_width = 600;
    const box_height = 250;

    return (
        <Center bg="gray.100" w={box_width} h={box_height} borderRadius={"md"}>
            <VStack spacing={4} align={"center"}>
                <AssetSelect type={type} onAssetChange={onTypeChange} />
                {/* asset is BTC: render BTC input*/}
                {type === AssetType.BTC
                    && <BitcoinInput amount={amount} onAmountChange={onAmountChange} />}
                {/* asset is USDT: render USDT input*/}
                {type === AssetType.USDT
                    && <UsdtInput amount={amount} onAmountChange={onAmountChange} />}
            </VStack>
        </Center>
    );
}

export default AssetSelector;

interface BitcoinInputProps {
    amount: number;
    onAmountChange: (amount: number) => void;
}

function BitcoinInput({ amount, onAmountChange }: BitcoinInputProps) {
    const format = (val: string) => {
        return `₿ ` + val;
    };

    const parse = (val: string) => {
        return val.replace(/^₿/, "");
    };

    const [value, setValue] = React.useState(amount.toString());

    const updateValue = (val: string) => {
        let asString = parse(val);
        setValue(asString);
        onAmountChange(Number(asString));
    };

    return (
        <NumberInput
            onChange={(valueString) => updateValue(valueString)}
            value={format(value)}
            precision={8}
            step={0.00000001}
            size="lg"
            bg="#FFFFFF"
            textStyle="actionable"
        >
            <NumberInputField />
        </NumberInput>
    );
}

interface UsdtInputProps {
    amount: number;
    onAmountChange: (amount: number) => void;
}

function UsdtInput({ amount, onAmountChange }: UsdtInputProps) {
    const format = (val: string) => {
        return `$ ` + val;
    };

    const parse = (val: string) => {
        return val.replace(/^\$/, "");
    };
    const [value, setValue] = React.useState(amount.toString());

    const updateValue = (val: string) => {
        let asString = parse(val);
        setValue(asString);
        onAmountChange(Number(asString));
    };

    return (
        <NumberInput
            onChange={(valueString) => updateValue(valueString)}
            value={format(value)}
            precision={8}
            step={0.01}
            size="lg"
            bg="#FFFFFF"
            textStyle="actionable"
        >
            <NumberInputField />
        </NumberInput>
    );
}
