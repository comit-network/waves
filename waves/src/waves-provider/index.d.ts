import { Address, CreateSwapPayload, LoanRequestPayload, LoanTx, Txid, WalletStatus } from "./wavesProvider";

declare global {
    interface Window {
        wavesProvider?: WavesProvider;
    }
}

export default class WavesProvider {
    public async walletStatus(): Promise<WalletStatus>;

    public async getSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload>;

    public async getBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload>;

    public async getNewAddress(): Promise<Address>;

    public async makeLoanRequestPayload(
        collateral: string,
        fee_rate: string,
        timeout: string,
    ): Promise<LoanRequestPayload>;

    public async signAndSendSwap(tx_hex: string): Promise<Txid>;

    public async signLoan(loan_response: any): Promise<LoanTx>;
}
