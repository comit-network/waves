import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { SwapToSign } from "../models";
import {
    createWallet,
    extractTrade,
    getAddress,
    getBalances,
    makeBuyCreateSwapPayload,
    makeLoanRequestPayload,
    makeSellCreateSwapPayload,
    signAndSendSwap,
    unlockWallet,
    walletStatus,
} from "../wasmProxy";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

const walletName = "demo";
var swapToSign: SwapToSign | undefined;

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
                const tabId = sender.tab!.id!;

                swapToSign = { txHex, decoded, tabId };
                break;
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
