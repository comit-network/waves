import { IconButton } from "@chakra-ui/react";
import React from "react";
import { TiArrowSync } from "react-icons/ti";

interface ExchangeIconProps {
    onClick: () => void;
    dataCy: string;
}
export default function ExchangeIcon({ onClick, dataCy }: ExchangeIconProps) {
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
            onClick={onClick}
            data-cy={dataCy}
        />
    );
}
