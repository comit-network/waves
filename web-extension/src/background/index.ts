import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { walletStatus } from "../wasmProxy";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

browser.runtime.onMessage.addListener(async (msg: Message<any>, sender) => {
    debug(
        `Received: "${JSON.stringify(msg)}" from tab ${sender.tab?.id}`,
    );

    if (msg.direction === Direction.ToBackground) {
        switch (msg.kind) {
            case MessageKind.WalletStatusRequest:
                const payload = await walletStatus();
                return { kind: MessageKind.WalletStatusResponse, direction: Direction.ToPage, payload };
        }
    }
});

function someMethodInBGPage() {
    return "Hello" + walletStatus;
}

// @ts-ignore
window.someMethodInBGPage = someMethodInBGPage;
