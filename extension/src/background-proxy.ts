import { browser } from "webextension-polyfill-ts";
import {
    Address,
    BackupDetails,
    BalanceUpdate,
    LoanDetails,
    LoanToSign,
    SwapToSign,
    Txid,
    WalletStatus,
} from "./models";

const proxy = browser.extension.getBackgroundPage();

export async function getAddress(): Promise<Address> {
    // @ts-ignore
    return proxy.getAddress();
}

export async function signAndSendSwap(txHex: string): Promise<void> {
    // @ts-ignore
    return proxy.signAndSendSwap(txHex);
}

export async function signLoan(): Promise<string> {
    // @ts-ignore
    return proxy.signLoan();
}

export async function confirmLoan(payload: string): Promise<void> {
    // @ts-ignore
    return proxy.confirmLoan(payload);
}

export async function createLoanBackup(loanTx: string): Promise<string> {
    // @ts-ignore
    return proxy.createLoanBackup(loanTx);
}

export async function loadLoanBackup(backupDetails: BackupDetails): Promise<void> {
    // @ts-ignore
    return proxy.loadLoanBackup(backupDetails);
}

export async function getLoanToSign(): Promise<LoanToSign | undefined> {
    // @ts-ignore
    return proxy.getLoanToSign();
}

export async function getSwapToSign(): Promise<SwapToSign | undefined> {
    // @ts-ignore
    return proxy.getSwapToSign();
}

export async function rejectLoan(): Promise<void> {
    // @ts-ignore
    return proxy.rejectLoan();
}

export async function rejectSwap(): Promise<void> {
    // @ts-ignore
    return proxy.rejectSwap();
}

export async function withdrawAll(address: string): Promise<Txid> {
    // @ts-ignore
    return proxy.withdrawAll(address);
}

export async function getWalletStatus(): Promise<WalletStatus> {
    // @ts-ignore
    return proxy.getWalletStatus();
}

export async function createWalletFromBip39(seed_words: string, password: string): Promise<void> {
    // @ts-ignore
    return proxy.createWalletFromBip39(seed_words, password);
}

export async function bip39SeedWords(): Promise<string> {
    // @ts-ignore
    return proxy.bip39SeedWords();
}

export async function unlockWallet(password: string): Promise<void> {
    // @ts-ignore
    return proxy.unlockWallet(password);
}

export async function getBalances(): Promise<BalanceUpdate> {
    // @ts-ignore
    return proxy.getBalances();
}

export async function getOpenLoans(): Promise<LoanDetails[]> {
    // @ts-ignore
    return proxy.getOpenLoans();
}

export async function repayLoan(txid: string): Promise<void> {
    // @ts-ignore
    return proxy.repayLoan(txid);
}

export async function getPastTransactions(): Promise<Txid[]> {
    // @ts-ignore
    return proxy.getPastTransactions();
}
