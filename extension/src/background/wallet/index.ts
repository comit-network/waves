import Debug from "debug";
import {
    Address,
    BackupDetails,
    BalanceUpdate,
    CreateSwapPayload,
    LoanDetails,
    LoanRequestPayload,
    Trade,
    Txid,
    WalletStatus,
} from "../../models";

Debug.enable("*");
const debug = Debug("wasmProxy");

export async function walletStatus(name: string): Promise<WalletStatus> {
    const { wallet_status } = await import("./generated");

    debug("walletStatus");
    return wallet_status(name);
}

export async function getAddress(name: string): Promise<Address> {
    const { get_address } = await import("./generated");

    debug("getAddress");
    return get_address(name);
}

export async function unlockWallet(name: string, password: string): Promise<void> {
    const { load_existing_wallet } = await import("./generated");

    debug("unlockWallet");
    return load_existing_wallet(name, password);
}

export async function getBalances(name: string): Promise<BalanceUpdate> {
    const { get_balances } = await import("./generated");

    debug("getBalances");
    return get_balances(name);
}

export async function makeSellCreateSwapPayload(name: string, btc: string): Promise<CreateSwapPayload> {
    const { make_sell_create_swap_payload } = await import("./generated");

    debug("makeSellCreateSwapPayload");
    return make_sell_create_swap_payload(name, btc);
}

export async function makeBuyCreateSwapPayload(name: string, usdt: string): Promise<CreateSwapPayload> {
    const { make_buy_create_swap_payload } = await import("./generated");

    debug("makeBuyCreateSwapPayload");
    return make_buy_create_swap_payload(name, usdt);
}

export async function makeLoanRequestPayload(
    name: string,
    collateral: string,
    fee_rate: string,
): Promise<LoanRequestPayload> {
    const { make_loan_request } = await import("./generated");

    debug("makeLoanRequestPayload");
    return make_loan_request(name, collateral, fee_rate);
}

export async function signAndSendSwap(name: string, hex: string): Promise<Txid> {
    const { sign_and_send_swap_transaction } = await import("./generated");

    debug("signAndSendSwap");

    const tx = { inner: hex };
    return sign_and_send_swap_transaction(name, tx);
}

export async function extractTrade(name: string, hex: string): Promise<Trade> {
    const { extract_trade } = await import("./generated");

    debug("extractTrade");
    const tx = { inner: hex };
    return extract_trade(name, tx);
}

// TODO: Replace any with actual LoanResponse interface
export async function extractLoan(name: string, loanResponse: any): Promise<LoanDetails> {
    const { extract_loan } = await import("./generated");

    debug("extractLoan");
    return extract_loan(name, loanResponse);
}

export async function signLoan(name: string): Promise<string> {
    const { sign_loan } = await import("./generated");

    debug("signLoan");
    return (await sign_loan(name)).inner;
}

export async function createLoanBackup(name: string, loanTx: string): Promise<BackupDetails> {
    const { create_loan_backup } = await import("./generated");

    debug("createLoanBackup");
    const tx = { inner: loanTx };
    return create_loan_backup(name, tx);
}

export async function loadLoanBackup(backupDetails: BackupDetails): Promise<void> {
    const { load_loan_backup } = await import("./generated");

    debug("loadLoanBackup");
    return load_loan_backup(backupDetails);
}

export async function withdrawAll(name: string, address: string): Promise<Txid> {
    const { withdraw_everything_to } = await import("./generated");

    debug("withdrawAll");
    return withdraw_everything_to(name, address);
}

export async function getOpenLoans(): Promise<LoanDetails[]> {
    const { get_open_loans } = await import("./generated");

    debug("getOpenLoans");
    return get_open_loans();
}

export async function repayLoan(name: string, txid: string): Promise<void> {
    const { repay_loan } = await import("./generated");

    debug("repayLoan");
    return repay_loan(name, txid);
}

export async function getPastTransactions(name: string): Promise<Txid[]> {
    const { get_past_transactions } = await import("./generated");

    debug("getPastTransactions");
    return get_past_transactions(name);
}

export async function bip39SeedWords(): Promise<string> {
    const { bip39_seed_words } = await import("./generated");

    debug("bip39_seed_words");
    return bip39_seed_words();
}

export async function createNewBip39Wallet(name: string, seedWords: string, password: string): Promise<string> {
    const { create_new_bip39_wallet } = await import("./generated");

    debug("create_new_bip39_wallet");
    return create_new_bip39_wallet(name, seedWords, password);
}
