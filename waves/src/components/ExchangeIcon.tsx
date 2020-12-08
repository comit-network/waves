import React from "react";
import { IconContext } from "react-icons";
import { TiArrowSync } from "react-icons/ti";

export default function ExchangeIcon() {
    return (
        <IconContext.Provider
            value={{
                color: "white",
                size: "60px",
                style: {
                    background: " #263238",
                    width: "64px",
                    height: "64px",
                    borderRadius: "50%",
                    textAlign: "center",
                    lineHeight: "100px",
                    verticalAlign: " middle",
                    padding: "10px",
                },
            }}
        >
            <TiArrowSync />
        </IconContext.Provider>
    );
}
