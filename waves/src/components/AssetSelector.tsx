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
    const box_width = 400;
    const box_height = 220;

    return (
        <Center bg="gray.100" w={box_width} h={box_height} borderRadius={"md"}>
            <VStack spacing={4} id="select{type}">
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
        <CustomInput value={value} precision={8} step={0.00000001} updateValue={updateValue} format={format} />
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
        <CustomInput value={value} precision={2} step={0.01} updateValue={updateValue} format={format} />
    );
}

interface CustomInputProps {
    value: string;
    precision: number;
    step: number;
    updateValue: (val: string) => void;
    format: (val: string) => string;
}

function CustomInput({ value, updateValue, precision, step, format }: CustomInputProps) {
    return (
        <NumberInput
            onChange={(valueString) => updateValue(valueString)}
            value={format(value)}
            precision={precision}
            step={step}
            size="lg"
            bg="#FFFFFF"
            textStyle="actionable"
            borderRadius={"md"}
            shadow="md"
        >
            <NumberInputField />
        </NumberInput>
    );
}
