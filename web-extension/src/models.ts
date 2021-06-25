export const USDT_TICKER = "USDT";
export const BTC_TICKER = "BTC";

export enum Status {
    None = "None",
    Loaded = "Loaded",
    NotLoaded = "NotLoaded",
}

export type Address = string;

export type WalletStatusRequest = {};

export interface WalletStatus {
    status: Status;
    address?: Address;
}

export interface BalanceEntry {
    assetId: string;
    ticker: string;
    value: number;
}

export type BalanceUpdate = Array<BalanceEntry>;

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

export interface SwapToSign {
    txHex: string;
    decoded: Trade;
    tabId: number;
}

export interface LoanToSign {
    txHex: string;
    collateral: TradeSide;
    principal: TradeSide;
    principalRepayment: number;
    term: number;
    tabId: number;
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
