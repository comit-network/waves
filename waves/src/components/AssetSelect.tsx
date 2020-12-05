import { ChevronDownIcon } from "@chakra-ui/icons";
import { Select } from "@chakra-ui/react";
import React from "react";
import { AssetType } from "../App";

interface CurrencySelectProps {
    type: AssetType;
    onAssetChange: (asset: AssetType) => void;
}
// TODO: make the select option nice as in the mock, i.e. with ticker symbols
function AssetSelect({ type, onAssetChange }: CurrencySelectProps) {
    const onChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        let value = event.target.value as keyof typeof AssetType;
        onAssetChange(AssetType[value]);
    };
    return (
        <>
            <Select
                size="lg"
                bg="#FFFFFF"
                textStyle="actionable"
                icon={<ChevronDownIcon />}
                iconColor="gray.500"
                defaultValue={type}
                onChange={onChange}
            >
                <option value={AssetType.BTC}>L-BTC - Bitcoin</option>
                <option value={AssetType.USDT}>L-USDT - Tether</option>
            </Select>
        </>
    );
}

export default AssetSelect;
