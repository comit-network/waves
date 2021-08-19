import debug from "debug";
import { Wallet } from "../background/api";
import { InvokeEventListenerViaContentScript } from "./impl";

declare let window: Window & {
    wavesProvider: Wallet;
};

window.wavesProvider = new InvokeEventListenerViaContentScript();

debug("inpage")("`wavesProvider` initialized");
