import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { helloWorld } from "../wasmProxy";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

helloWorld();

browser.runtime.onMessage.addListener(async (msg, sender) => {
    debug(
        "Message from the content script: "
            + msg.greeting,
    );
    return { response: "Response from Background script" };
});
