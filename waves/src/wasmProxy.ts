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

const WALLET_NAME = "wallet-1";

export async function getAddress() {
    // const { get_address } = await import("./wallet");
    // return get_address(WALLET_NAME);
    return "dummyAddress";
}

export async function newWallet(password: string): Promise<WalletStatus> {
    // const { create_new_wallet } = await import("./wallet");
    // return create_new_wallet(WALLET_NAME, password).then(getWalletStatus);
    return Promise.resolve({
        loaded: false,
        exists: false,
    });
}

export async function unlockWallet(password: string) {
    // const { load_existing_wallet } = await import("./wallet");
    // return load_existing_wallet(WALLET_NAME, password).then(getWalletStatus);
}

export async function lockWallet() {
    // const { unload_current_wallet } = await import("./wallet");
    // return unload_current_wallet().then(getWalletStatus);
}

export async function getWalletStatus(): Promise<WalletStatus> {
    // @ts-ignore
    if (!window.wallet_status) {
        debug("wallet_status not found. CS not yet defined? ");
        return {
            loaded: false,
            exists: false,
        };
    }
    // @ts-ignore
    debug("Retrieving wallet status");
    // @ts-ignore
    return await window.wallet_status();
}

export async function getBalances(): Promise<BalanceEntry[]> {
    // TODO create bindings for library
    // @ts-ignore
    if (!window.balances) {
        return [];
    }
    debug("Retrieving balances");
    // @ts-ignore
    return await window.balances();
}

export async function withdrawAll(address: string): Promise<String> {
    // const { withdraw_everything_to } = await import("./wallet");
    // return withdraw_everything_to(WALLET_NAME, address);
    return Promise.resolve("");
}

export async function makeSellCreateSwapPayload(
    btc: string,
): Promise<CreateSwapPayload> {
    // @ts-ignore
    if (!window.get_sell_create_swap_payload) {
        return Promise.reject();
    }
    debug("making sell create swap payload");
    // @ts-ignore
    return await window.get_sell_create_swap_payload(btc);
}

export async function makeBuyCreateSwapPayload(
    usdt: string,
): Promise<CreateSwapPayload> {
    // const { make_buy_create_swap_payload } = await import("./wallet");
    // return make_buy_create_swap_payload(WALLET_NAME, usdt);
    return Promise.resolve({
        alice_inputs: [],
        address: "dummyAddress",
        amount: 1,
    });
}

export async function extractTrade(
    transaction: string,
): Promise<Trade> {
    // const { extract_trade } = await import("./wallet");
    // return extract_trade(WALLET_NAME, transaction);
    debug("Transaction from bobtimus: " + transaction);

    return Promise.resolve({
        sell: {
            ticker: Asset.LBTC,
            amount: 0,
            balanceBefore: 0,
            balanceAfter: 0,
        },
        buy: {
            ticker: Asset.USDT,
            amount: 0,
            balanceBefore: 0,
            balanceAfter: 0,
        },
    });
}

export async function signAndSend(
    transaction: string,
): Promise<string> {
    // const {sign_and_send_swap_transaction} = await import("./wallet");
    // return sign_and_send_swap_transaction(WALLET_NAME, transaction);
    return Promise.resolve("tx");
}
