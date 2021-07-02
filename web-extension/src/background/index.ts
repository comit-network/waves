import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { LoanToSign, SwapToSign } from "../models";
import {
    createWallet,
    extractLoan,
    extractTrade,
    getAddress,
    getBalances,
    makeBuyCreateSwapPayload,
    makeLoanRequestPayload,
    makeSellCreateSwapPayload,
    signAndSendSwap,
    signLoan,
    unlockWallet,
    walletStatus,
    withdrawAll,
} from "../wasmProxy";

// TODO: Is this global or do we need one per file?
Debug.enable("*");
const debug = Debug("background");

debug("Hello world from background script");

const walletName = "demo";
var swapToSign: SwapToSign | undefined;
var loanToSign: LoanToSign | undefined;

browser.runtime.onMessage.addListener(async (msg: Message<any>, sender) => {
    debug(
        `Received: "${JSON.stringify(msg)}" from tab ${sender.tab?.id}`,
    );

    if (msg.direction === Direction.ToBackground) {
        let payload;
        let kind;
        switch (msg.kind) {
            case MessageKind.WalletStatusRequest:
                payload = await walletStatus(walletName);
                kind = MessageKind.WalletStatusResponse;
                break;
            case MessageKind.SellRequest:
                const btc = msg.payload;
                payload = await makeSellCreateSwapPayload(walletName, btc);
                kind = MessageKind.SellResponse;
                break;
            case MessageKind.BuyRequest:
                const usdt = msg.payload;
                payload = await makeBuyCreateSwapPayload(walletName, usdt);
                kind = MessageKind.BuyResponse;
                break;
            case MessageKind.AddressRequest:
                payload = await getAddress(walletName);
                kind = MessageKind.AddressResponse;
                break;
            case MessageKind.LoanRequest:
                const collateral = msg.payload;
                payload = await makeLoanRequestPayload(walletName, collateral);
                kind = MessageKind.LoanResponse;
                break;
            case MessageKind.SignAndSendSwap:
                const txHex = msg.payload;
                const decoded = await extractTrade(walletName, txHex);

                swapToSign = { txHex, decoded, tabId: sender.tab!.id! };
                return;
            case MessageKind.SignLoan:
                const loanResponse = msg.payload;
                const details = await extractLoan(walletName, loanResponse);

                loanToSign = { details, tabId: sender.tab!.id! };
                return;
        }
        return { kind, direction: Direction.ToPage, payload };
    }
});

// @ts-ignore
window.createWallet = async (password: string) => {
    await createWallet(walletName, password);
};
// @ts-ignore
window.getWalletStatus = async () => {
    return walletStatus(walletName);
};
// @ts-ignore
window.unlockWallet = async (password: string) => {
    await unlockWallet(walletName, password);
};
// @ts-ignore
window.getBalances = async () => {
    return getBalances(walletName);
};
// @ts-ignore
window.getAddress = async () => {
    return getAddress(walletName);
};
// @ts-ignore
window.getSwapToSign = async () => {
    return swapToSign;
};
// @ts-ignore
window.signAndSendSwap = async (txHex: string, tabId: number) => {
    const txid = await signAndSendSwap(walletName, txHex);
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SwapTxid, payload: txid });
    swapToSign = undefined;

    return txid;
};
// @ts-ignore
window.rejectSwap = async (tabId: number) => {
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SwapRejected });
    swapToSign = undefined;
};
// @ts-ignore
window.getLoanToSign = async () => {
    return loanToSign;
};
// @ts-ignore
window.signLoan = async (tabId: number) => {
    // TODO: Currently, we assume that whatever the user has verified
    // on the pop-up matches what is stored in the extension's
    // storage. It would be better to send around the swap ID to check
    // that the wallet is signing the same transaction the user has authorised

    const loan = await signLoan(walletName);
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SignedLoan, payload: loan });
    loanToSign = undefined;
};
// @ts-ignore
window.rejectLoan = async (tabId: number) => {
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.LoanRejected });
    loanToSign = undefined;
};
// @ts-ignore
window.withdrawAll = async (address: string) => {
    return withdrawAll(walletName, address);
};
