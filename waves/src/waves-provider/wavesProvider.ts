import { Asset } from "../App";

export interface BalanceEntry {
    asset: string;
    value: number;
    ticker?: string;
}

export type Address = string;

export type Txid = string;

export type LoanTx = string;

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
