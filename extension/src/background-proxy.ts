import { browser } from "webextension-polyfill-ts";
import { Address, BalanceUpdate, LoanDetails, LoanToSign, SwapToSign, Txid, WalletStatus } from "./models";

const proxy = browser.extension.getBackgroundPage();

export async function getAddress(): Promise<Address> {
    // @ts-ignore
    return proxy.getAddress();
}

export async function signAndSendSwap(txHex: string, tabId: number): Promise<Txid> {
    // @ts-ignore
    return proxy.signAndSendSwap(txHex, tabId);
}

export async function signLoan(tabId: number): Promise<void> {
    // @ts-ignore
    return proxy.signLoan(tabId);
}

export async function getLoanToSign(): Promise<LoanToSign | undefined> {
    // @ts-ignore
    return proxy.getLoanToSign();
}

export async function getSwapToSign(): Promise<SwapToSign | undefined> {
    // @ts-ignore
    return proxy.getSwapToSign();
}

export async function rejectLoan(tabId: number): Promise<void> {
    // @ts-ignore
    return proxy.rejectLoan(tabId);
}

export async function rejectSwap(tabId: number): Promise<void> {
    // @ts-ignore
    return proxy.rejectSwap(tabId);
}

export async function withdrawAll(address: string): Promise<Txid> {
    // @ts-ignore
    return proxy.withdrawAll(address);
}

export async function getWalletStatus(): Promise<WalletStatus> {
    // @ts-ignore
    return proxy.getWalletStatus();
}

export async function createWallet(password: string): Promise<void> {
    // @ts-ignore
    return proxy.createWallet(password);
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

export async function repayLoan(_txId: String): Promise<void> {
    return;
}

export async function getPastTransactions(): Promise<Txid[]> {
    return Promise.resolve(
        [
            "590b5979a7273d70a481ffe016da98a67826008c6364b56d24bb49399e3c274a",
            "8f88e92d0ec41347b5f10d5665983093b777bc4026f703935e639598ad8d2237",
        ],
    );
}
