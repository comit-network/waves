import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { helloWorld } from "../wasmProxy";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

helloWorld();

let state = "This state";

browser.runtime.onMessage.addListener(async (msg, sender) => {
    debug(
        `Received: "${msg.message}" from ${sender.tab?.id}`,
    );
    state = msg.message;

    return { response: "Response from Background script" };
});

function someMethodInBGPage() {
    return "Hello" + state;
}

// @ts-ignore
window.someMethodInBGPage = someMethodInBGPage;
