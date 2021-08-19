import { Address, CreateSwapPayload, LoanRequestPayload, LoanTx, Txid, WalletStatus } from "./wavesProvider";

declare global {
    interface Window {
        wavesProvider?: Wallet;
    }
}

// This needs to match `Wallet` from `extension/background/api.ts`
export interface Wallet {
    walletStatus(): Promise<WalletStatus>;
    makeSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload>;
    makeBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload>;
    getNewAddress(): Promise<Address>;
    makeLoanRequestPayload(
        collateral: string,
        fee_rate: string,
    ): Promise<LoanRequestPayload>;
    requestSignSwap(tx_hex: string): Promise<Txid>;
    requestSignLoan(loan_response: any): Promise<LoanTx>;
}
