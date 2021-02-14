import Debug from "debug";
import React, { ReactElement } from "react";
import { SSEProvider } from "react-hooks-sse";
import { CreateSwapPayload } from "./wasmProxy";

const debug = Debug("bobtimus");

export async function fundAddress(address: string): Promise<any> {
    await fetch("/api/faucet/" + address, {
        method: "POST",
    });
}

export async function postSellPayload(payload: CreateSwapPayload) {
    return await postPayload(payload, "sell");
}

export async function postBuyPayload(payload: CreateSwapPayload) {
    return await postPayload(payload, "buy");
}

async function postPayload(payload: CreateSwapPayload, path: string) {
    let res = await fetch(`/api/swap/lbtc-lusdt/${path}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
        body: JSON.stringify(payload),
    });

    if (res.status !== 200) {
        debug("failed to create new swap");
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
