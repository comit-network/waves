import Debug from "debug";
import { Address, Status, WalletStatus } from "./models";

Debug.enable("*");
let debug = Debug("wasm-proxy");

export async function walletStatus(): Promise<WalletStatus> {
    const { wallet_status } = await import("./wallet");

    debug("walletStatus");
    const status = await wallet_status();

    if (status.loaded && status.exists) {
        let address = await getAddress();
        return { status: Status.Loaded, address };
    } else if (status.exists) {
        return { status: Status.NotLoaded };
    } else {
        return { status: Status.None };
    }
}

export async function getAddress(): Promise<Address> {
    const { get_address } = await import("./wallet");

    debug("getAddress");
    return await get_address();
}
