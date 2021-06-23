import Debug from "debug";
import { browser } from "webextension-polyfill-ts";

Debug.enable("content");
const debug = Debug("content");

debug("Hello world from content script");

async function notifyBackgroundPage(message: string): Promise<string> {
    try {
        debug(`Sending: "${message}"`);
        const response = await browser.runtime.sendMessage({
            message: message,
        });
        debug(`Response: "${response.response}"`);
        return response.response;
    } catch (error) {
        debug(`Error: ${error}`);
        throw error;
    }
}

window.addEventListener("message", async function(event) {
    if (
        event.source === window
        && event.data
        && event.data.direction === "from-page-script"
    ) {
        debug("Received: \"" + event.data.message + "\"");

        let response = await notifyBackgroundPage(event.data.message);
        window.postMessage({
            direction: "from-content-script",
            message: response,
        }, "*");
    }
});

/**
 * Injects a script tag into the current document
 *
 * @param {string} contentPath - Path to be js file to be included
 */
async function injectScript(contentPath: string) {
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

(async function() {
    await injectScript(inpageUrl);
}());
