import Debug from "debug";
import { browser } from "webextension-polyfill-ts";
import { Direction, Message, MessageKind } from "../messages";
import { Address, BalanceUpdate, WalletStatus } from "../models";
import {
    createWallet as create,
    getAddress as address,
    getBalances as balances,
    unlockWallet as unlock,
    walletStatus,
} from "../wasmProxy";

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

// @ts-ignore
window.createWallet = async (password: string) => {
    await create(walletName, password);
};

// @ts-ignore
window.getWalletStatus = async () => {
    return walletStatus(walletName);
};
// @ts-ignore
window.unlockWallet = async (password: string) => {
    await unlock(walletName, password);
};
// @ts-ignore
window.getBalances = async () => {
    return balances(walletName);
};
// @ts-ignore
window.getAddress = async () => {
    return address(walletName);
};
