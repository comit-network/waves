// TODO: Try to delete this layer or at least rename the file

import Debug from "debug";
import { Asset } from "./App";
const debug = Debug("wavesProviderProxy");
const error = Debug("wavesProviderProxy:error");

export interface BalanceEntry {
    asset: string;
    value: number;
    ticker?: string;
}

export interface WalletStatus {
    status: Status;
    address?: string;
}

export enum Status {
    None = "None",
    Loaded = "Loaded",
    NotLoaded = "NotLoaded",
}

export interface CreateSwapPayload {
    alice_inputs: { outpoint: OutPoint; blinding_key: string }[];
    address: string;
    amount: number;
}

export interface LoanRequestPayload {
    collateral_amount: number;
    // TODO: Replace `any` with concrete type or get rid of `original_txout` field
    collateral_inputs: { txin: OutPoint; original_txout: any; blinding_key: string }[];
    fee_sats_per_vbyte: number;
    borrower_pk: string;
    timelock: number;
    borrower_address: string;
}

export interface OutPoint {
    txid: string;
    vout: number;
}

export interface Trade {
    sell: TradeSide;
    buy: TradeSide;
}

export interface TradeSide {
    ticker: Asset;
    amount: number;
    balanceBefore: number;
    balanceAfter: number;
}

export async function getWalletStatus(): Promise<WalletStatus> {
    // @ts-ignore
    if (!window.wavesProvider.walletStatus) {
        error("walletStatus undefined");
        return Promise.reject("walletStatus undefined");
    }
    // @ts-ignore
    return window.wavesProvider.walletStatus();
}

export async function makeSellCreateSwapPayload(
    btc: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.wavesProvider.getSellCreateSwapPayload) {
        return Promise.reject("getSellCreateSwapPayload undefined");
    }
    // @ts-ignore
    return await window.wavesProvider.getSellCreateSwapPayload(btc);
}

export async function makeBuyCreateSwapPayload(
    usdt: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.wavesProvider.getBuyCreateSwapPayload) {
        return Promise.reject("getBuyCreateSwapPayload undefined");
    }
    // @ts-ignore
    return await window.wavesProvider.getBuyCreateSwapPayload(usdt);
}

export async function signAndSend(
    transaction: string,
): Promise<string> {
    // @ts-ignore
    if (!window.wavesProvider.signAndSendSwap) {
        return Promise.reject("signAndSendSwap undefined");
    }

    // @ts-ignore
    return await window.wavesProvider.signAndSendSwap(transaction);
}

export async function getNewAddress(): Promise<string> {
    // @ts-ignore
    if (!window.wavesProvider.getNewAddress) {
        return Promise.reject("getNewAddress undefined");
    }

    // @ts-ignore
    return await window.wavesProvider.getNewAddress();
}

export async function makeLoanRequestPayload(collateral: string): Promise<LoanRequestPayload> {
    // @ts-ignore
    if (!window.wavesProvider.makeLoanRequestPayload) {
        return Promise.reject("makeLoanRequestPayload undefined");
    }
    // @ts-ignore
    return await window.wavesProvider.makeLoanRequestPayload(collateral);
}

export async function signLoan(loanResponse: any): Promise<any> {
    // @ts-ignore
    if (!window.wavesProvider.signLoan) {
        return Promise.reject("signLoan undefined");
    }
    // @ts-ignore
    return await window.wavesProvider.signLoan(loanResponse);
}
