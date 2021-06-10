import { Center, VStack } from "@chakra-ui/react";
import React, { Dispatch } from "react";
import { Action, Asset, AssetSide } from "../App";
import AssetSelect from "./AssetSelect";
import NumberInput from "./NumberInput";

type StringOrNumber = string | number;

interface AssetSelectorProps {
    assetSide: AssetSide;
    type: Asset;
    amount: StringOrNumber;
    placement: "left" | "right";
    dispatch: Dispatch<Action>;
}

function AssetSelector({ assetSide, type, amount, placement, dispatch }: AssetSelectorProps) {
    const box_width = 400;
    const box_height = 220;

    const onAmountChange = (newAmount: string) => {
        switch (assetSide) {
            case "Alpha":
                dispatch({
                    type: "UpdateAlphaAmount",
                    value: newAmount,
                });
                break;
            default:
                throw new Error("Only support editing alpha amount at the moment");
        }
    };

    const onAssetTypeChange = (newType: Asset) => {
        switch (assetSide) {
            case "Alpha":
                dispatch({
                    type: "UpdateAlphaAssetType",
                    value: newType,
                });
                break;
            case "Beta":
                dispatch({
                    type: "UpdateBetaAssetType",
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
                {type === Asset.LBTC
                    && <NumberInput
                        currency="₿"
                        value={amount}
                        precision={7}
                        step={0.000001}
                        onAmountChange={onAmountChange}
                        isDisabled={assetSide === "Beta"}
                        dataCy="data-cy-L-BTC"
                    />}
                {/* asset is USDT: render USDT input*/}
                {type === Asset.USDT
                    && <NumberInput
                        currency="$"
                        value={amount}
                        precision={2}
                        step={0.01}
                        onAmountChange={onAmountChange}
                        isDisabled={assetSide === "Beta"}
                        dataCy="data-cy-USDt"
                    />}
            </VStack>
        </Center>
    );
}

export default AssetSelector;
