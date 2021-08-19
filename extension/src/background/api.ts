import { AsyncReturnType } from "type-fest";
import { browser } from "webextension-polyfill-ts";
import { ParametersObject } from "../type-utils";

// Represents the API of our wallet that is accessible from within privileged contexts like the popup window or other background scripts.
export type BackgroundWindow = BackgroundWindowWasm & BackgroundWindowTypescript;

// Functionality and state added to the background window from within WebAssembly.
interface BackgroundWindowWasm {
    extractTrade(hex: string): Promise<Trade>;
    extractLoan(loanRequest: LoanRequestPayload): Promise<LoanDetails>;
    signAndSendSwap(hex: string): Promise<Txid>;
    unlockWallet(password: string): Promise<void>;
    withdrawAll(address: string): Promise<Txid>;
    getWalletStatus(): Promise<WalletStatus>;
    getBalances(): Promise<BalanceEntry[]>;
    createNewWallet(seedWords: string, password: string): Promise<void>;
    repayLoan(txid: string): Promise<Txid>;
    getAddress(): Promise<string>;
    signLoan(): Promise<string>; // TODO: It is weird that we are calling this without any parameters.

    // TODO: Implement these in Typescript instead.
    getOpenLoans(): Promise<LoanDetails[]>;
    createLoanBackup(hex: string): Promise<any>;
    loadLoanBackup(backup: any): Promise<void>;
    generateBip39SeedWords(): Promise<string>;
}

// Functionality and state added to the background window from within Typescript.
export interface BackgroundWindowTypescript {
    swapToSign: SwapToSign | null;
    loanToSign: LoanToSign | null;
    approveSwap(): Promise<void>;
    rejectSwap(): Promise<void>;
    approveLoan(): Promise<string>;
    rejectLoan(): Promise<void>;
    publishLoan(tx: string): Promise<void>;
}

// Represents the API of our wallet that is accessible from non-privileged contexts like the content script and as a result of that, the web page.
export type Wallet = EventListenersWasm & EventListenersTypescript;

interface EventListenersWasm {
    walletStatus(): Promise<WalletStatus>;
    getNewAddress(): Promise<Address>;
    makeSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload>;
    makeBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload>;
    makeLoanRequestPayload(collateral: string, fee_rate: string): Promise<LoanRequestPayload>;
}

export interface EventListenersTypescript {
    requestSignSwap(hex: string): Promise<Txid>;
    requestSignLoan(loanRequest: LoanRequestPayload): Promise<string>;
}

// Access the background page directly.
//
// This is only available from within other privileges scopes but allows convenient access to _state_ of the wallet.
// Functionality should still be accessed via RPC calls.
export function backgroundPage(): Promise<BackgroundWindow> {
    return browser.runtime.getBackgroundPage() as unknown as Promise<BackgroundWindow>;
}

export async function invokeEventListener<T extends keyof Wallet>(
    message: Omit<RpcMessage<T>, "type">,
): Promise<AsyncReturnType<Wallet[T]>> {
    const result = await browser.runtime.sendMessage({
        type: "rpc-message",
        ...message,
    }) as RpcResponse<T>;

    if ("Ok" in result) {
        return result.Ok;
    }

    if ("Err" in result) {
        throw new Error(result.Err);
    }

    throw new Error("result object has unexpected shape");
}

export interface RpcMessage<T extends keyof Wallet> {
    type: "rpc-message";
    method: T;
    args: ParametersObject<Wallet[T]>;
}

export interface SwapToSign {
    txHex: string;
    decoded: Trade;
}

export interface LoanToSign {
    details: LoanDetails;
}

export type RpcResponse<T extends keyof Wallet> = {
    Ok: AsyncReturnType<Wallet[T]>;
} | {
    Err: any;
};

export const USDT_TICKER = "L-USDt";
export const BTC_TICKER = "L-BTC";

export interface WalletStatus {
    status: Status;
    address?: Address;
}

export enum Status {
    None = "None",
    Loaded = "Loaded",
    NotLoaded = "NotLoaded",
}

export type Address = string;

export interface BalanceEntry {
    assetId: string;
    ticker: string;
    value: number;
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

export interface LoanRequestPayload {
    collateral_amount: number;
    // TODO: Replace `any` with concrete type or get rid of `original_txout` field
    collateral_inputs: { txin: OutPoint; original_txout: any; blinding_key: string }[];
    fee_sats_per_vbyte: number;
    borrower_pk: string;
    timelock: number;
    borrower_address: string;
}

export type Txid = string;

export interface TradeSide {
    ticker: string;
    amount: number;
    balanceBefore: number;
    balanceAfter: number;
}

export interface Trade {
    buy: TradeSide;
    sell: TradeSide;
}

export interface LoanDetails {
    collateral: TradeSide;
    principal: TradeSide;
    principalRepayment: number;
    term: number;
    txid: Txid;
}
