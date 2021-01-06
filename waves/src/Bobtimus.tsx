import React, { ReactElement } from "react";
import { SSEProvider } from "react-hooks-sse";
import { CreateSwapPayload } from "./wasmProxy";

export async function fundAddress(address: string): Promise<any> {
    await fetch("/api/faucet/" + address, {
        method: "POST",
    });
}

export async function postSellPayload(payload: CreateSwapPayload) {
    let res = await fetch("/api/swap/lbtc-lusdt/sell", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
        body: JSON.stringify(payload),
    });

    if (res.status !== 200) {
        console.error("failed to create new swap"); // TODO: use logger instead of console
        throw new Error("failed to create new swap");
    }

    return await res.text();
}

interface RateProviderProps {
    children: ReactElement;
}

export function BobtimusRateProvider({ children }: RateProviderProps) {
    return <SSEProvider endpoint="/api/rate/lbtc-lusdt">{children}</SSEProvider>;
}
