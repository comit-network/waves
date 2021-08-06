import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { LoanDetails, LoanToSign, SwapToSign } from "../models";
import {
    bip39SeedWords,
    createNewBip39Wallet,
    extractLoan,
    extractTrade,
    getAddress,
    getBalances,
    getBlockHeight,
    getOpenLoans,
    getPastTransactions,
    makeBuyCreateSwapPayload,
    makeLoanRequestPayload,
    makeSellCreateSwapPayload,
    repayLoan,
    signAndSendSwap,
    signLoan,
    unlockWallet,
    walletStatus,
    withdrawAll,
} from "../wasmProxy";

// TODO: Is this global or do we need one per file?
Debug.enable("*");
const debug = Debug("background");
const error = Debug("background:error");

// First thing we load settings
loadSettings();

debug("Hello world from background script");

const walletName = "demo";
var swapToSign: SwapToSign | undefined;
var loanToSign: LoanToSign | undefined;

browser.runtime.onMessage.addListener(async (msg: Message<any>, sender) => {
    debug(`Received: %o from tab %d`, msg, sender.tab?.id);

    if (msg.direction === Direction.ToBackground) {
        let message;
        switch (msg.kind) {
            case MessageKind.WalletStatusRequest:
                message = await call_wallet(() => walletStatus(walletName), MessageKind.WalletStatusResponse);
                break;
            case MessageKind.SellRequest:
                message = await call_wallet(
                    async () => await makeSellCreateSwapPayload(walletName, msg.payload),
                    MessageKind.SellResponse,
                );
                break;
            case MessageKind.BuyRequest:
                message = await call_wallet(
                    async () => await makeBuyCreateSwapPayload(walletName, msg.payload),
                    MessageKind.BuyResponse,
                );
                break;
            case MessageKind.AddressRequest:
                message = await call_wallet(
                    async () => await getAddress(walletName),
                    MessageKind.AddressResponse,
                );
                break;
            case MessageKind.LoanRequest:
                message = await call_wallet(
                    async () =>
                        await makeLoanRequestPayload(
                            walletName,
                            msg.payload.collateral,
                            msg.payload.fee_rate,
                        ),
                    MessageKind.LoanResponse,
                );
                break;
            case MessageKind.SignAndSendSwap:
                try {
                    const txHex = msg.payload;
                    const decoded = await extractTrade(walletName, txHex);
                    swapToSign = { txHex, decoded, tabId: sender.tab!.id! };
                    updateBadge();
                } catch (e) {
                    error(e);
                    message = { kind: MessageKind.SwapTxid, direction: Direction.ToPage, error: e };
                }
                break;
            case MessageKind.SignLoan:
                try {
                    const details = await extractLoan(walletName, msg.payload);
                    loanToSign = { details, tabId: sender.tab!.id! };
                    updateBadge();
                } catch (e) {
                    error(e);
                    message = { kind: MessageKind.SignedLoan, direction: Direction.ToPage, error: e };
                }
                break;
        }
        return message;
    }
});

async function call_wallet<T>(wallet_fn: () => Promise<T>, kind: MessageKind): Promise<Message<T | undefined>> {
    let payload;
    let err;
    try {
        payload = await wallet_fn();
    } catch (e) {
        error(e);
        err = e;
    }

    return { kind, direction: Direction.ToPage, payload, error: err };
}

// @ts-ignore
window.getWalletStatus = async () => {
    return walletStatus(walletName);
};
// @ts-ignore
window.unlockWallet = async (password: string) => {
    return unlockWallet(walletName, password);
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
    let payload;
    let err;

    try {
        payload = await signAndSendSwap(walletName, txHex);
    } catch (e) {
        error(e);
        err = e;
    }

    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SwapTxid, payload, error: err });
    swapToSign = undefined;
    updateBadge();
};
// @ts-ignore
window.rejectSwap = (tabId: number) => {
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SwapRejected });
    swapToSign = undefined;
    updateBadge();
};
// @ts-ignore
window.getLoanToSign = () => {
    return loanToSign;
};
// @ts-ignore
window.signLoan = async (tabId: number) => {
    // TODO: Currently, we assume that whatever the user has verified
    // on the pop-up matches what is stored in the extension's
    // storage. It would be better to send around the swap ID to check
    // that the wallet is signing the same transaction the user has authorised

    let payload;
    let err;

    try {
        payload = await signLoan(walletName);
    } catch (e) {
        error(e);
        err = e;
    }

    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.SignedLoan, payload, error: err });
    loanToSign = undefined;
    updateBadge();
};
// @ts-ignore
window.rejectLoan = (tabId: number) => {
    browser.tabs.sendMessage(tabId, { direction: Direction.ToPage, kind: MessageKind.LoanRejected });
    loanToSign = undefined;
    updateBadge();
};
// @ts-ignore
window.withdrawAll = async (address: string) => {
    return withdrawAll(walletName, address);
};
// @ts-ignore
window.getOpenLoans = async (): LoanDetails[] => {
    return getOpenLoans();
};
// @ts-ignore
window.repayLoan = async (txid: string): void => {
    return repayLoan(walletName, txid);
};
// @ts-ignore
window.getPastTransactions = async (): Txid[] => {
    return getPastTransactions(walletName);
};
// @ts-ignore
window.bip39SeedWords = async (): string => {
    return bip39SeedWords();
};
// @ts-ignore
window.createWalletFromBip39 = async (seed_words: string, password: string) => {
    return createNewBip39Wallet(walletName, seed_words, password);
};

// @ts-ignore
window.getBlockHeight = async () => {
    return getBlockHeight();
};

function updateBadge() {
    let count = 0;
    if (loanToSign) count++;
    if (swapToSign) count++;
    browser.browserAction.setBadgeText(
        { text: (count === 0 ? null : count.toString()) },
    );
}

function loadSettings() {
    debug("Loading settings");
    ensureVarSet("ESPLORA_API_URL");
    ensureVarSet("CHAIN");
    ensureVarSet("LBTC_ASSET_ID");
    ensureVarSet("LUSDT_ASSET_ID");
}

// First we check environment variable. If set, we honor it and overwrite settings in local storage.
// For the environment variable we add the prefix `REACT_APP_`.
// Else we check browser storage. If set, we honor it, if not, we throw an error as we cannot work
// without this value.
function ensureVarSet(name: string) {
    const uppercase = name.toUpperCase();
    const value = process.env[`REACT_APP_${uppercase}`];
    if (value) {
        debug(`Environment variable provided, overwriting storage: ${name}:${value}`);
        localStorage.setItem(name, value);
    }
}
