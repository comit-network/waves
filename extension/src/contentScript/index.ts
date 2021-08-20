import Debug from "debug";
import { AsyncReturnType } from "type-fest";
import { browser } from "webextension-polyfill-ts";
import { invokeEventListener, Wallet } from "../background/api";
import { ParametersObject } from "../type-utils";

Debug.enable("*");
const debug = Debug("content");

debug("Hello world from content script");

export interface RpcRequest<T extends keyof Wallet> {
    type: "wallet-rpc-request";
    id: string;
    method: T;
    args: ParametersObject<Wallet[T]>;
}

export interface RpcResponse<T extends keyof Wallet> {
    type: "wallet-rpc-response";
    id: string;
    ok?: AsyncReturnType<Wallet[T]>;
    err?: string;
}

window.addEventListener("message", (event: MessageEvent<RpcRequest<keyof Wallet>>) => {
    if (event.source === window && event.data.type === "wallet-rpc-request") {
        invokeEventListener({
            method: event.data.method,
            args: event.data.args,
        }).then(ok => {
            let responseMessage: RpcResponse<keyof Wallet> = {
                type: "wallet-rpc-response",
                id: event.data.id,
                ok,
            };

            window.postMessage(responseMessage, "*");
        }).catch(err => {
            let responseMessage: RpcResponse<keyof Wallet> = {
                type: "wallet-rpc-response",
                id: event.data.id,
                err: err.toString(),
            };

            window.postMessage(responseMessage, "*");
        });
    }
});

/**
 * Injects a script tag into the current document
 *
 * @param {string} contentPath - Path to be js file to be included
 */
function injectScript(contentPath: string) {
    try {
        const container = document.head || document.documentElement;
        const scriptTag = document.createElement("script");
        scriptTag.setAttribute("async", "false");
        scriptTag.setAttribute("src", contentPath);
        container.insertBefore(scriptTag, container.children[0]);
    } catch (error) {
        console.error("WavesExtension: Provider injection failed.", error);
    }
}

const inpageUrl = browser.runtime.getURL("in_page.bundle.js");

injectScript(inpageUrl);
