import Debug from "debug";
import React, { ReactElement } from "react";
import { SSEProvider } from "react-hooks-sse";
import { CreateSwapPayload, LoanRequestPayload, OutPoint } from "./waves-provider/wavesProvider";

const debug = Debug("bobtimus");
const BTC_SATS = 100000000;

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

export interface Term {
    days: number;
    // percentage, decimal represented as float
    // example:
    // 0.01 => add 0.01 to base interest
    // -0.01 => subtract 0.01 from base interest
    interest_mod: number;
}

export interface Collateralization {
    // percentage, decimal represented as float
    // example:
    // 1.5 => 150%
    collateralization: number;
    // percentage, decimal represented as float
    // example:
    // 0.01 => add 0.01 to base interest
    // -0.01 => subtract 0.01 from base interest
    interest_mod: number;
}

export interface LoanOffer {
    rate: Rate;
    fee_sats_per_vbyte: number;
    min_principal: number; // sat
    max_principal: number; // sat
    // percentage, decimal represented as float
    // 0.8 => 80%
    max_ltv: number;
    base_interest_rate: number;
    terms: Term[];
    collateralizations: Collateralization[];
}

export interface LoanRequest {
    principal_amount: number;
    collateral_amount: number;
    collateral_inputs: { txin: OutPoint; original_txout: any; blinding_key: string }[];
    collateralization: number;
    borrower_pk: string;
    borrower_address: string;

    /// Loan term in days
    term: number;
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

export async function postLoanRequest(
    walletParams: LoanRequestPayload,
    termInDays: number,
    collateralization: number,
    principal: number,
) {
    // TODO: Make sure to convert all the other amounts to sats as well
    // convert principal to sats
    let principal_sats = principal * BTC_SATS;

    let loanRequest: LoanRequest = {
        collateralization: collateralization,
        principal_amount: principal_sats,
        borrower_address: walletParams.borrower_address,
        borrower_pk: walletParams.borrower_pk,
        collateral_amount: walletParams.collateral_amount,
        collateral_inputs: walletParams.collateral_inputs,
        term: termInDays,
    };

    let res = await fetch(`/api/loan/lbtc-lusdt`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
        },
        body: JSON.stringify(loanRequest),
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
