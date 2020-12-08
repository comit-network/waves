import { IconButton } from "@chakra-ui/react";
import React, { Dispatch } from "react";
import { TiArrowSync } from "react-icons/ti";
import { UpdateAssetAction } from "../App";

interface ExchangeIconProps {
    dispatch: Dispatch<UpdateAssetAction>;
}
export default function ExchangeIcon({ dispatch }: ExchangeIconProps) {
    return (
        <IconButton
            variant="solid"
            aria-label="Swap"
            fontSize="20px"
            isRound
            bg="#263238"
            width="64px"
            height="64px"
            _hover={{ bg: "rgba(38,50,56,0.68)" }}
            icon={<TiArrowSync size="40px" color="white" />}
            onClick={() =>
                dispatch({
                    type: "SwapAssetTypes",
                })}
        />
    );
}
