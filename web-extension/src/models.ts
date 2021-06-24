export const USDT_TICKER = "USDT";
export const BTC_TICKER = "BTC";

export enum Status {
    None,
    Loaded,
    NotLoaded,
}

export type Address = string;

export interface WalletStatus {
    status: Status;
}

export interface BalanceEntry {
    assetId: string;
    ticker: string;
    value: number;
}

export interface BalanceUpdate {
    balances: Array<BalanceEntry>;
}

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
