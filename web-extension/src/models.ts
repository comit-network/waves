export const USDT_TICKER = "USDT";
export const BTC_TICKER = "BTC";

export enum Status {
    None = "None",
    Loaded = "Loaded",
    NotLoaded = "NotLoaded",
}

export type Address = string;

export type Txid = string;

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

export interface LoanDetails {
    collateral: TradeSide;
    principal: TradeSide;
    principalRepayment: number;
    term: number;
}

export interface LoanToSign {
    details: LoanDetails;
    tabId: number;
}

export type LoanTx = string;

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
