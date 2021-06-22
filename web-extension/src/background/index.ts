import Debug from "debug";
import { helloWorld } from "../wasmProxy";
import { browser } from "webextension-polyfill-ts";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

helloWorld();

browser.runtime.onMessage.addListener(async (msg, sender) => {
    debug("Received message", msg, "from tab ID", sender.tab?.id);
});
