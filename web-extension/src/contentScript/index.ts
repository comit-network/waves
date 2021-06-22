import Debug from "debug";
import { browser } from "webextension-polyfill-ts";

Debug.enable("content");
const debug = Debug("content");

debug("Hello world from content script");

browser.runtime.sendMessage(`Hello world from content script on tab: ${window.location.hostname}`);

const inpageUrl = browser.runtime.getURL("in_page.bundle.js");

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

injectScript(inpageUrl);
