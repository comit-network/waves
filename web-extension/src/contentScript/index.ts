import Debug from "debug";
import { browser } from "webextension-polyfill-ts";

Debug.enable("content");
const debug = Debug("content");

debug("Hello world from content script");

browser.runtime.sendMessage(`Hello world from content script on tab: ${window.location.hostname}`);

