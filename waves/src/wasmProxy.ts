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

export interface LoanRequestPayload {
    dummy_field: string;
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
    if (typeof window.liquid === "undefined") {
        debug("wallet_status not found. CS not yet defined? ");
        return Promise.reject("wallet_status undefined");
    }
    // @ts-ignore
    return await window.liquid.wallet_status();
}

export async function makeSellCreateSwapPayload(
    btc: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.liquid.get_sell_create_swap_payload) {
        return Promise.reject("get_sell_create_swap_payload undefined");
    }
    // @ts-ignore
    return await window.liquid.get_sell_create_swap_payload(btc);
}

export async function makeBuyCreateSwapPayload(
    usdt: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.liquid.get_buy_create_swap_payload) {
        return Promise.reject("get_buy_create_swap_payload undefined");
    }
    // @ts-ignore
    return await window.liquid.get_buy_create_swap_payload(usdt);
}

export async function signAndSend(
    transaction: string,
): Promise<string> {
    // @ts-ignore
    if (!window.liquid.sign_and_send) {
        return Promise.reject("sign_and_send undefined");
    }

    // @ts-ignore
    return await window.liquid.sign_and_send(transaction);
}

export async function getNewAddress(): Promise<string> {
    // @ts-ignore
    if (!window.liquid.new_address) {
        return Promise.reject("new_address undefined");
    }

    // @ts-ignore
    return await window.liquid.new_address();
}

export async function makeBorrowPayload(principal_amount: string): Promise<LoanRequestPayload> {
    // @ts-ignore
    if (!window.liquid.get_loan_request_payload) {
        return Promise.reject("get_loan_request_payload undefined");
    }
    // @ts-ignore
    return await window.liquid.get_loan_request_payload(principal_amount);
}
