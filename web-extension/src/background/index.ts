import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { WalletStatus } from "../models";
import { walletStatus, createWallet as create, unlockWallet as unlock } from "../wasmProxy";

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

export async function getWalletStatus(): Promise<WalletStatus> {
    return await walletStatus(walletName);
}

export async function createWallet(password: string): Promise<void> {
    await create(walletName, password);
}

export async function unlockWallet(password: string): Promise<void> {
    await unlock(walletName, password);
}

// @ts-ignore
window.createWallet = createWallet;
// @ts-ignore
window.getWalletStatus = getWalletStatus;
// @ts-ignore
window.unlockWallet = unlockWallet;
