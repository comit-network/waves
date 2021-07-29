import Debug from "debug";
import React, { ReactElement } from "react";
import { SSEProvider } from "react-hooks-sse";
import { CreateSwapPayload, LoanRequestPayload } from "./waves-provider/wavesProvider";

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

export interface Rate {
    ask: number; // sat
    bid: number; // sat
}

export interface Interest {
    timelock: number;
    interest_rate: number; // percentage, decimal represented as float
}

export interface LoanOffer {
    rate: Rate;
    fee_sats_per_vbyte: number;
    min_principal: number; // sat
    max_principal: number; // sat
    max_ltv: number; // percentage, decimal represented as float
    interest: Interest[];
}

export async function getLoanOffer(): Promise<LoanOffer> {
    let res = await fetch(`/api/loan/lbtc-lusdt`, {
        method: "GET",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
    });

    if (res.status !== 200) {
        debug("failed to fetch loan offer");
        throw new Error("failed to fetch loan offer");
    }

    return await res.json();
}

export async function postLoanRequest(payload: LoanRequestPayload) {
    let res = await fetch(`/api/loan/lbtc-lusdt`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
        body: JSON.stringify(payload),
    });

    if (res.status !== 200) {
        debug("failed to create new loan");
        throw new Error("failed to create new loan");
    }

    return await res.json();
}

export async function postLoanFinalization(txHex: string) {
    let res = await fetch(`/api/loan/lbtc-lusdt/finalize`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
        body: JSON.stringify({ tx_hex: txHex }),
    });

    if (res.status !== 200) {
        debug("failed to create new loan");
        throw new Error("failed to create new loan");
    }

    return await res.json();
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
