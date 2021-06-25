import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { BalanceUpdate, WalletStatus } from "../models";
import { createWallet as create, getBalances as balances, unlockWallet as unlock, walletStatus } from "../wasmProxy";

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

export async function getBalances(): Promise<BalanceUpdate> {
    return await balances(walletName);
}

// @ts-ignore
window.createWallet = createWallet;
// @ts-ignore
window.getWalletStatus = getWalletStatus;
// @ts-ignore
window.unlockWallet = unlockWallet;
