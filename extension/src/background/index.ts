import debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { ParametersObject } from "../type-utils";
import { BackgroundWindow, BackgroundWindowTypescript, EventListenersTypescript, RpcMessage, RpcResponse } from "./api";

// Define the global fields on our background window
declare let window: BackgroundWindow;

const log = debug("background-script");

loadSettings();
initializeEventListeners();
initializeWindowExtensions();
initializeWasmModule();

function loadSettings() {
    ensureVarSet("ESPLORA_API_URL");
    ensureVarSet("CHAIN");
    ensureVarSet("LBTC_ASSET_ID");
    ensureVarSet("LUSDT_ASSET_ID");

    log("Settings loaded");
}

function initializeEventListeners() {
    addRpcMessageListener("requestSignSwap", ({ hex }) => {
        return new Promise(resolve => {
            window.extractTrade(hex)
                .then(decoded => {
                    window.swapToSign = { txHex: hex, decoded };
                    resolveSwapSignRequest = resolve;

                    return updateBadge();
                })
                .catch(e => {
                    resolve({ Err: e });
                    return cleanupPendingSwap();
                });
        });
    });

    // @ts-ignore: Why does this not work?
    addRpcMessageListener("requestSignLoan", ({ loanRequest }) => {
        return new Promise(resolve => {
            void window.extractLoan(loanRequest)
                .then(details => {
                    window.loanToSign = { details };
                    resolveLoanSignRequest = resolve;

                    return updateBadge();
                })
                .catch(e => {
                    resolve({ Err: e });
                    return cleanupPendingLoan();
                });
        });
    });

    log("Typescript event listeners initialized");
}

function addRpcMessageListener<T extends keyof EventListenersTypescript>(
    method: T,
    callback: (args: ParametersObject<EventListenersTypescript[T]>) => Promise<RpcResponse<T>>,
) {
    browser.runtime.onMessage.addListener((msg: RpcMessage<T>) => {
        if (msg.type !== "rpc-message" || msg.method !== method) {
            return;
        }

        log(`Received: %o`, msg);

        return callback(msg.args);
    });
}

function initializeWindowExtensions() {
    const windowExt: BackgroundWindowTypescript = {
        swapToSign: null,
        loanToSign: null,
        approveSwap: async () => {
            if (!resolveSwapSignRequest || !window.swapToSign) {
                throw new Error("No pending promise function for swap sign request");
            }

            try {
                const txid = await window.signAndSendSwap(window.swapToSign.txHex);
                resolveSwapSignRequest({ Ok: txid });
            } catch (e) {
                resolveSwapSignRequest({ Err: e });
            } finally {
                await cleanupPendingSwap();
            }
        },
        rejectSwap: () => {
            if (!resolveSwapSignRequest) {
                throw new Error("No pending promise function for swap sign request");
            }

            resolveSwapSignRequest({ Err: "User declined signing request" });
            return cleanupPendingSwap();
        },
        approveLoan: () => {
            if (!resolveLoanSignRequest || !window.loanToSign) {
                throw new Error("No pending promise function for loan sign request");
            }

            return window.signLoan();
        },
        rejectLoan: () => {
            if (!resolveLoanSignRequest) {
                throw new Error("No pending promise function for loan sign request");
            }

            resolveLoanSignRequest({ Err: "User declined signing request" });
            return cleanupPendingLoan();
        },
        publishLoan: async (tx: string) => {
            if (!resolveLoanSignRequest) {
                throw new Error("No pending promise function for loan sign request");
            }
            // once sent to the page, we assume the business is done.
            // TODO: a feedback loop is required where the wallet gets told if bobtimus successfully published the transaction
            resolveLoanSignRequest({ Ok: tx });
            await cleanupPendingLoan();
        },
    };

    Object.assign(window, windowExt);

    log("Typescript window extensions initialized");
}

function initializeWasmModule() {
    void import("./wallet/generated").then(wallet => {
        wallet.initialize();
        log("WASM module initialized");
    });
}

// Private fields of the background script
var resolveSwapSignRequest: ((response: RpcResponse<"requestSignSwap">) => void) | null;
var resolveLoanSignRequest: ((response: RpcResponse<"requestSignLoan">) => void) | null;

// First we check environment variable. If set, we honor it and overwrite settings in local storage.
// For the environment variable we add the prefix `REACT_APP_`.
// Else we check browser storage. If set, we honor it, if not, we throw an error as we cannot work
// without this value.
function ensureVarSet(name: string) {
    const uppercase = name.toUpperCase();
    const value = process.env[`REACT_APP_${uppercase}`];
    if (value) {
        log(`Environment variable provided, overwriting storage: ${name}:${value}`);
        localStorage.setItem(name, value);
    }
}

function updateBadge() {
    let count = 0;
    if (window.loanToSign) count++;
    if (window.swapToSign) count++;

    return browser.browserAction.setBadgeText(
        { text: (count === 0 ? null : count.toString()) },
    );
}

function cleanupPendingSwap() {
    resolveSwapSignRequest = null;
    window.swapToSign = null;
    return updateBadge();
}

function cleanupPendingLoan() {
    resolveLoanSignRequest = null;
    window.loanToSign = null;
    return updateBadge();
}
