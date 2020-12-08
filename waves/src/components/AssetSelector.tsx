import {
    Center,
    InputGroup,
    InputLeftAddon,
    NumberInput as CUINumberInput,
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
            default:
                throw new Error("Only support editing alpha amount at the moment");
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
                    && <NumberInput
                        currency="₿"
                        value={amount}
                        precision={7}
                        step={0.000001}
                        updateValue={onAmountChange}
                        isDisabled={assetSide === "Beta"}
                    />}
                {/* asset is USDT: render USDT input*/}
                {type === AssetType.USDT
                    && <NumberInput
                        currency="$"
                        value={amount}
                        precision={2}
                        step={0.01}
                        updateValue={onAmountChange}
                        isDisabled={assetSide === "Beta"}
                    />}
            </VStack>
        </Center>
    );
}

export default AssetSelector;

interface CustomInputProps {
    currency: string;
    value: number;
    precision: number;
    step: number;
    updateValue: (val: number) => void;
    isDisabled: boolean;
}

const ASSET_INPUT_LEFT_ADDON_PROPS = {
    size: "lg",
    textStyle: "actionable",
    w: "15%",
    h: "3rem",
    bg: "grey.50",
    borderRadius: "md",
    shadow: "md",
};

const ASSET_INPUT_PROPS = {
    w: "100%",
    size: "lg",
    bg: "#FFFFFF",
    textStyle: "actionable",
    borderRadius: "md",
    shadow: "md",
};

const ASSET_INPUT_DISABLED_PROPS = {
    ...ASSET_INPUT_PROPS,
    bg: "grey.50",
};

function NumberInput({ currency, value, updateValue, precision, step, isDisabled }: CustomInputProps) {
    const inputProps = isDisabled ? ASSET_INPUT_DISABLED_PROPS : ASSET_INPUT_PROPS;
    return (
        <InputGroup>
            <InputLeftAddon
                {...ASSET_INPUT_LEFT_ADDON_PROPS}
                children={currency}
            />
            <CUINumberInput
                {...inputProps}
                onChange={(_, valueNumber) => updateValue(valueNumber)}
                value={value}
                precision={precision}
                step={step}
                isDisabled={isDisabled}
                min={0}
            >
                <NumberInputField />
                <NumberInputStepper />
            </CUINumberInput>
        </InputGroup>
    );
}
