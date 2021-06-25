import Debug from "debug";
import { Address, BalanceUpdate, CreateSwapPayload, Status, WalletStatus } from "./models";

Debug.enable("*");
let debug = Debug("wasm-proxy");

export async function walletStatus(name: string): Promise<WalletStatus> {
    const { wallet_status } = await import("./wallet");

    debug("walletStatus");
    const status = await wallet_status(name);

    if (status.loaded && status.exists) {
        let address = await getAddress(name);
        return { status: Status.Loaded, address };
    } else if (status.exists) {
        return { status: Status.NotLoaded };
    } else {
        return { status: Status.None };
    }
}

export async function getAddress(name: string): Promise<Address> {
    const { get_address } = await import("./wallet");

    debug("getAddress");
    return await get_address(name);
}

export async function createWallet(name: string, password: string): Promise<null> {
    const { create_new_wallet } = await import("./wallet");

    debug("createWallet");
    return await create_new_wallet(name, password);
}

export async function unlockWallet(name: string, password: string): Promise<null> {
    const { unlock_wallet } = await import("./wallet");

    debug("unlockWallet");
    return await unlock_wallet(name, password);
}

export async function getBalances(name: string): Promise<BalanceUpdate> {
    const { get_balances } = await import("./wallet");

    debug("getBalances");
    return await get_balances(name);
}

export async function makeSellCreateSwapPayload(name: string, btc: string): Promise<CreateSwapPayload> {
    const { make_sell_create_swap_payload } = await import("./wallet");

    debug("makeSellCreateSwapPayload");
    return await make_sell_create_swap_payload(name, btc);
}

export async function makeBuyCreateSwapPayload(name: string, usdt: string): Promise<CreateSwapPayload> {
    const { make_buy_create_swap_payload } = await import("./wallet");

    debug("makeBuyCreateSwapPayload");
    return await make_buy_create_swap_payload(name, usdt);
}

export async function makeLoanRequestPayload(name: string, collateral: string): Promise<CreateSwapPayload> {
    const { make_loan_request_payload } = await import("./wallet");

    debug("makeLoanRequestPayload");
    return await make_loan_request_payload(name, collateral);
}
