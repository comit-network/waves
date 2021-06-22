import Debug from "debug";
import { browser } from "webextension-polyfill-ts";

Debug.enable("content");
const debug = Debug("content");

debug("Hello world from content script");

// browser.runtime.sendMessage(`Hello world from content script on tab: ${window.location.hostname}`);

async function notifyBackgroundPage() {
    try {
        debug("Sending message to background page");
        const response = await browser.runtime.sendMessage({
            greeting: "Greeting from the content script",
        });
        debug(`Response:  ${response.response}`);
    } catch (error) {
        debug(`Error: ${error}`);
    }
}

window.addEventListener("message", async function(event) {
    if (
        event.source === window
        && event.data
        && event.data.direction === "from-page-script"
    ) {
        debug("Content script received message: \"" + event.data.message + "\"");

        await notifyBackgroundPage();
        return "Success";
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
