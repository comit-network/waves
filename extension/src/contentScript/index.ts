import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message } from "../messages";

Debug.enable("*");
const debug = Debug("content");

debug("Hello world from content script");

async function forwardToBackground(message: Message<any>): Promise<Message<any>> {
    try {
        return await browser.runtime.sendMessage(message);
    } catch (error) {
        debug(`Error: ${JSON.stringify(error)}`);
        throw error;
    }
}

window.addEventListener("message", async function(event: MessageEvent<Message<any>>) {
    if (
        event.source === window
        && event.data.direction === Direction.ToBackground
    ) {
        debug(`Forwarding request from ips to bs: ${JSON.stringify(event.data)}`);
        let response = await forwardToBackground(event.data);

        if (response) {
            debug(`Forwarding response from bs to ips: ${JSON.stringify(response)}`);
            window.postMessage(response, "*");
        }
    }
});

browser.runtime.onMessage.addListener(async function(msg: Message<any>) {
    // Some messages from the background script (the ones that depend on
    // user interaction via the pop-up), are not a direct response to a
    // message we send fro the content script, so we must be ready to
    // listen for messages from the background script and forward them to
    // the content script too
    if (msg.direction === Direction.ToPage) {
        debug(`Forwarding message from bs to ips: ${JSON.stringify(msg)}`);
        window.postMessage(msg, "*");
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
