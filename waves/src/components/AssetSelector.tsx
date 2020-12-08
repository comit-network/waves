import {
    Center,
    InputGroup,
    InputLeftAddon,
    NumberInput,
    NumberInputField,
    NumberInputStepper,
    VStack,
} from "@chakra-ui/react";
import React, { Dispatch } from "react";
import { AssetSide, AssetType, UpdateAssetAction } from "../App";
import AssetSelect from "./AssetSelect";

interface AssetSelectorProps {
    assetSide: AssetSide;
    type: AssetType;
    amount: number;
    placement: "left" | "right";
    dispatch: Dispatch<UpdateAssetAction>;
}

function AssetSelector({ assetSide, type, amount, placement, dispatch }: AssetSelectorProps) {
    const box_width = 400;
    const box_height = 220;

    const onAmountChange = (newAmount: number) => {
        switch (assetSide) {
            case "Alpha":
                dispatch({
                    type: "AlphaAmount",
                    value: newAmount,
                });
                break;
            case "Beta":
                dispatch({
                    type: "BetaAmount",
                    value: newAmount,
                });
                break;
            default:
                throw new Error("Unknown asset side");
        }
    };

    const onAssetTypeChange = (newType: AssetType) => {
        switch (assetSide) {
            case "Alpha":
                dispatch({
                    type: "AlphaAssetType",
                    value: newType,
                });
                break;
            case "Beta":
                dispatch({
                    type: "BetaAssetType",
                    value: newType,
                });
                break;
            default:
                throw new Error("Unknown asset side");
        }
    };

    return (
        <Center bg="gray.100" w={box_width} h={box_height} borderRadius={"md"}>
            <VStack spacing={4} id="select{type}">
                <AssetSelect type={type} onAssetChange={onAssetTypeChange} placement={placement} />
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

interface InputProps {
    amount: number;
    onAmountChange: (amount: number) => void;
}

function BitcoinInput({ amount, onAmountChange }: InputProps) {
    return (
        <CustomInput currency="₿" value={amount} precision={8} step={0.00000001} updateValue={onAmountChange} />
    );
}

function UsdtInput({ amount, onAmountChange }: InputProps) {
    return (
        <CustomInput currency="$" value={amount} precision={2} step={0.01} updateValue={onAmountChange} />
    );
}

interface CustomInputProps {
    currency: string;
    value: number;
    precision: number;
    step: number;
    updateValue: (val: number) => void;
}

function CustomInput({ currency, value, updateValue, precision, step }: CustomInputProps) {
    return (
        <InputGroup>
            <InputLeftAddon
                children={currency}
                w="15%"
                size="lg"
                h="3rem"
                bg="grey.50"
                textStyle="actionable"
                borderRadius={"md"}
                shadow="md"
            />
            <NumberInput
                onChange={(_, valueNumber) => updateValue(valueNumber)}
                w="100%"
                value={value}
                precision={precision}
                step={step}
                size="lg"
                bg="#FFFFFF"
                textStyle="actionable"
                borderRadius={"md"}
                shadow="md"
                inputMode="decimal"
            >
                <NumberInputField />
                <NumberInputStepper />
            </NumberInput>
        </InputGroup>
    );
}
