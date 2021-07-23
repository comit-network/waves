import { browser } from "webextension-polyfill-ts";
import { Address, BalanceUpdate, LoanDetails, LoanToSign, SwapToSign, Txid, WalletStatus } from "../models";

const proxy = browser.extension.getBackgroundPage();

export async function getBalances(): Promise<BalanceUpdate> {
    // @ts-ignore
    return proxy.getBalances();
}
