import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import WavesProvider from "../in-page";
import { BackupDetails, LoanDetails, LoanToSign, SwapToSign, Txid } from "../models";
import {
    bip39_seed_words,
    create_loan_backup,
    create_new_bip39_wallet,
    extract_loan,
    extract_trade,
    get_address,
    get_balances,
    get_block_height,
    get_open_loans,
    get_past_transactions,
    load_loan_backup,
    make_buy_create_swap_payload,
    make_loan_request,
    make_sell_create_swap_payload,
    repay_loan,
    sign_and_send_swap_transaction,
    sign_loan,
    wallet_status,
    withdraw_everything_to,
} from "../wallet";

// TODO: Is this global or do we need one per file?
Debug.enable("*");
const debug = Debug("background");

// First thing we load settings
loadSettings();

debug("Hello world from background script");

const walletName = "demo";

var swapToSign: SwapToSign | null;
var resolveSwapSignRequest: ((txid: Txid) => void) | null;
var rejectSwapSignRequest: ((e: any) => void) | null;

var loanToSign: LoanToSign | null;
var resolveLoanSignRequest: ((txid: Txid) => void) | null;
var rejectLoanSignRequest: ((e: any) => void) | null;

export interface RpcMessage<T extends keyof WavesProvider> {
    type: "rpc-message";
    method: T;
    args: Parameters<WavesProvider[T]>;
}

/*
 * Defines the public interface of the background script.
 *
 * To ensure maximum benefit from the type checker, other components like the content script should only use this function to send messages.
 */
export function invokeBackgroundScriptRpc(message: Omit<RpcMessage<keyof WavesProvider>, "type">): Promise<any> {
    return browser.runtime.sendMessage({
        type: "rpc-message",
        ...message,
    });
}

addRpcMessageListener("walletStatus", () => wallet_status(walletName));
addRpcMessageListener("getBuyCreateSwapPayload", ([usdt]) => make_buy_create_swap_payload(walletName, usdt));
addRpcMessageListener("getSellCreateSwapPayload", ([btc]) => make_sell_create_swap_payload(walletName, btc));
addRpcMessageListener("getNewAddress", () => get_address(walletName));
addRpcMessageListener(
    "makeLoanRequestPayload",
    ([collateral, feerate]) => make_loan_request(walletName, collateral, feerate),
);

addRpcMessageListener("signAndSendSwap", ([txHex]) => {
    return new Promise((resolve, reject) => {
        extract_trade(walletName, txHex)
            .then(decoded => {
                swapToSign = { txHex, decoded };
                resolveSwapSignRequest = resolve;
                rejectSwapSignRequest = reject;

                updateBadge();
            })
            .catch(e => {
                reject(e);
                cleanupPendingSwap();
            });
    });
});
addRpcMessageListener("signLoan", ([loanRequest]) => {
    return new Promise((resolve, reject) => {
        extract_loan(walletName, loanRequest)
            .then(details => {
                loanToSign = { details };
                resolveLoanSignRequest = resolve;
                rejectLoanSignRequest = reject;

                updateBadge();
            })
            .catch(e => {
                reject(e);
                cleanupPendingLoan();
            });
    });
});

function addRpcMessageListener<T extends keyof WavesProvider>(
    method: T,
    callback: (args: Parameters<WavesProvider[T]>) => ReturnType<WavesProvider[T]>,
) {
    browser.runtime.onMessage.addListener((msg: RpcMessage<T>) => {
        if (msg.type !== "rpc-message" || msg.method !== method) {
            return;
        }

        debug(`Received: %o`, msg);

        return callback(msg.args);
    });
}

// @ts-ignore
window.getWalletStatus = async () => {
    return wallet_status(walletName);
};
// @ts-ignore
window.unlockWallet = async (password: string) => {
    return unlockWallet(walletName, password);
};
// @ts-ignore
window.getBalances = async () => {
    return get_balances(walletName);
};
// @ts-ignore
window.getAddress = async () => {
    return get_address(walletName);
};
// @ts-ignore
window.getSwapToSign = async () => {
    return swapToSign;
};
// @ts-ignore
window.signAndSendSwap = (txHex: string) => {
    if (!resolveSwapSignRequest || !rejectSwapSignRequest) {
        throw new Error("No pending promise functions for swap sign request");
    }

    sign_and_send_swap_transaction(walletName, txHex)
        .then(resolveSwapSignRequest)
        .catch(rejectSwapSignRequest)
        .then(cleanupPendingSwap);
};
// @ts-ignore
window.rejectSwap = () => {
    if (!resolveSwapSignRequest || !rejectSwapSignRequest) {
        throw new Error("No pending promise functions for swap sign request");
    }

    rejectSwapSignRequest("User declined signing request");
    cleanupPendingSwap();
};
// @ts-ignore
window.getLoanToSign = () => {
    return loanToSign;
};
// @ts-ignore
window.signLoan = async () => {
    if (!resolveLoanSignRequest || !rejectLoanSignRequest) {
        throw new Error("No pending promise functions for loan sign request");
    }

    // TODO: Currently, we assume that whatever the user has verified
    // on the pop-up matches what is stored in the extension's
    // storage. It would be better to send around the swap ID to check
    // that the wallet is signing the same transaction the user has authorised

    // if we receive an error, we respond directly, else we return the details
    return await sign_loan(walletName).catch(rejectLoanSignRequest);
};

// @ts-ignore
window.confirmLoan = async (payload: string) => {
    if (!resolveLoanSignRequest || !rejectLoanSignRequest) {
        throw new Error("No pending promise functions for loan sign request");
    }
    // once sent to the page, we assume the business is done.
    // TODO: a feedback loop is required where the wallet gets told if bobtimus successfully published the transaction
    resolveLoanSignRequest(payload);
    await cleanupPendingLoan();
};

// @ts-ignore
window.createLoanBackup = async (loanTx: string) => {
    return create_loan_backup(walletName, loanTx);
};

// @ts-ignore
window.loadLoanBackup = async (backupDetails: BackupDetails) => {
    return loan_loan_backup(backupDetails);
};

// @ts-ignore
window.rejectLoan = () => {
    if (!resolveLoanSignRequest || !rejectLoanSignRequest) {
        throw new Error("No pending promise functions for loan sign request");
    }

    rejectLoanSignRequest("User declined signing request");
    cleanupPendingLoan();
};
// @ts-ignore
window.withdrawAll = async (address: string) => {
    return withdraw_everything_to(walletName, address);
};
// @ts-ignore
window.getOpenLoans = async (): LoanDetails[] => {
    return get_open_loans();
};
// @ts-ignore
window.repayLoan = async (txid: string): void => {
    return repay_loan(walletName, txid);
};
// @ts-ignore
window.getPastTransactions = async (): Txid[] => {
    return get_past_transaction(walletName);
};
// @ts-ignore
window.bip39SeedWords = async (): string => {
    return bip39_seed_words();
};
// @ts-ignore
window.createWalletFromBip39 = async (seed_words: string, password: string) => {
    return create_new_bip39_wallet(walletName, seed_words, password);
};

// @ts-ignore
window.getBlockHeight = async () => {
    return get_block_height();
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

function cleanupPendingSwap() {
    resolveSwapSignRequest = null;
    rejectSwapSignRequest = null;
    swapToSign = null;
    updateBadge();
}

function cleanupPendingLoan() {
    resolveLoanSignRequest = null;
    rejectLoanSignRequest = null;
    loanToSign = null;
    updateBadge();
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
