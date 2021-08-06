import Debug from "debug";
import { AsyncReturnType } from "type-fest";
import { browser } from "webextension-polyfill-ts";
import { invokeBackgroundScriptRpc } from "../background";
import WavesProvider from "../in-page";

Debug.enable("*");
const debug = Debug("content");

debug("Hello world from content script");

export interface RequestMessage<T extends keyof WavesProvider> {
    type: "request";
    method: T;
    args: Parameters<WavesProvider[T]>;
    id: string;
}

export interface ResponseMessage<T extends keyof WavesProvider> {
    type: "response";
    id: string;
    ok?: AsyncReturnType<WavesProvider[T]>;
    err?: string;
}

window.addEventListener("message", (event: MessageEvent<RequestMessage<keyof WavesProvider>>) => {
    if (event.source !== window || event.data.type !== "request") {
        return;
    }

    invokeBackgroundScriptRpc({
        method: event.data.method,
        args: event.data.args,
    }).then(ok => {
        let responseMessage: ResponseMessage<keyof WavesProvider> = {
            type: "response",
            id: event.data.id,
            ok,
        };

        window.postMessage(responseMessage, "*");
    }).catch(err => {
        let responseMessage: ResponseMessage<keyof WavesProvider> = {
            type: "response",
            id: event.data.id,
            err: err.toString(), // Unfortunately, we have to send a string representation here
        };

        window.postMessage(responseMessage, "*");
    });
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
