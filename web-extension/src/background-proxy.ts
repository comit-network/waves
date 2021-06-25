import { browser } from "webextension-polyfill-ts";
import { Address, BalanceUpdate, BTC_TICKER, LoanToSign, SwapToSign, USDT_TICKER, WalletStatus } from "./models";

const proxy = browser.extension.getBackgroundPage();

export async function getAddress(): Promise<Address> {
    // @ts-ignore
    return proxy.getAddress();
}

export async function signAndSend(tx: string): Promise<string> {
    return Promise.resolve("8ec2ff513cb55b621af73130818c359aef357038905b7954775eff43e92916f9");
}

export async function getLoanToSign(): Promise<LoanToSign | undefined> {
    return Promise.resolve(undefined);
}

export async function getSwapToSign(): Promise<SwapToSign | undefined> {
    return Promise.resolve(undefined);
}

export async function cancelLoan(_loanToSign: LoanToSign): Promise<void> {
    return Promise.resolve();
}

export async function cancelSwap(_swapToSign: SwapToSign): Promise<void> {
    return Promise.resolve();
}

export async function withdrawAll(_address: string): Promise<void> {
    return Promise.resolve();
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
