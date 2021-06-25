import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { WalletStatus } from "../models";
import { createNewWallet, walletStatus } from "../wasmProxy";

Debug.enable("background");
const debug = Debug("background");

debug("Hello world from background script");

const walletName = "demo";

browser.runtime.onMessage.addListener(async (msg: Message<any>, sender) => {
    debug(
        `Received: "${JSON.stringify(msg)}" from tab ${sender.tab?.id}`,
    );

    if (msg.direction === Direction.ToBackground) {
        switch (msg.kind) {
            case MessageKind.WalletStatusRequest:
                const payload = await walletStatus(walletName);
                return { kind: MessageKind.WalletStatusResponse, direction: Direction.ToPage, payload };
        }
    }
});

export async function createWallet(password: string): Promise<void> {
    debug("Create wallet");

    await createNewWallet(walletName, password);

    return;
}

export async function getWalletStatus(): Promise<WalletStatus> {
    debug("Get wallet status");

    return await walletStatus(walletName);
}

// @ts-ignore
window.createWallet = createWallet;
