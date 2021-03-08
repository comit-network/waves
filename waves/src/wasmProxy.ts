import Debug from "debug";
import { Asset } from "./App";
const debug = Debug("wasm-proxy");

export interface BalanceEntry {
    asset: string;
    value: number;
    ticker?: string;
}

export interface WalletStatus {
    loaded: boolean;
    exists: boolean;
}

export interface CreateSwapPayload {
    alice_inputs: { outpoint: OutPoint; blinding_key: string }[];
    address: string;
    amount: number;
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
    if (typeof window.wallet_status === "undefined") {
        debug("wallet_status not found. CS not yet defined? ");
        return Promise.reject("wallet_status undefined");
    }
    // @ts-ignore
    debug("Retrieving wallet status " + typeof window.wallet_status);
    // @ts-ignore
    return await window.wallet_status();
}

export async function makeSellCreateSwapPayload(
    btc: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.get_sell_create_swap_payload) {
        return Promise.reject("get_sell_create_swap_payload undefined");
    }
    debug("making sell create swap payload");
    // @ts-ignore
    return await window.get_sell_create_swap_payload(btc);
}

export async function makeBuyCreateSwapPayload(
    usdt: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.get_buy_create_swap_payload) {
        return Promise.reject("get_buy_create_swap_payload undefined");
    }
    debug("making buy create swap payload");
    // @ts-ignore
    return await window.get_buy_create_swap_payload(usdt);
}

export async function signAndSend(
    transaction: string,
): Promise<String> {
    // @ts-ignore
    if (!window.sign_and_send) {
        return Promise.reject("sign_and_send undefined");
    }
    debug("signing and sending");
    // @ts-ignore
    return await window.sign_and_send(transaction);
}
