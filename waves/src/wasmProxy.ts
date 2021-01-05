export interface BalanceEntry {
    asset: string;
    value: number;
    ticker?: string;
}

export interface WalletStatus {
    loaded: boolean;
    exists: boolean;
}

export interface CreateSwapPayload {
    alice_inputs: { txid: string; vout: number }[];
    address_redeem: string;
    address_change: string;
    fee: number;
    btc_amount: number;
}

const WALLET_NAME = "wallet-1";

export async function getAddress() {
    const { get_address } = await import("./wallet");
    return get_address(WALLET_NAME);
}

export async function newWallet(password: string): Promise<WalletStatus> {
    const { create_new_wallet } = await import("./wallet");
    return create_new_wallet(WALLET_NAME, password).then(getWalletStatus);
}

export async function unlockWallet(password: string) {
    const { load_existing_wallet } = await import("./wallet");
    return load_existing_wallet(WALLET_NAME, password).then(getWalletStatus);
}

export async function lockWallet() {
    const { unload_current_wallet } = await import("./wallet");
    return unload_current_wallet().then(getWalletStatus);
}

export async function getWalletStatus(): Promise<WalletStatus> {
    const { wallet_status } = await import("./wallet");
    return wallet_status(WALLET_NAME);
}

export async function getBalances(): Promise<BalanceEntry[]> {
    const { get_balances } = await import("./wallet");
    return get_balances(WALLET_NAME);
}

export async function withdrawAll(address: string): Promise<String> {
    const { withdraw_everything_to } = await import("./wallet");
    return withdraw_everything_to(WALLET_NAME, address);
}

export async function makeCreateSellSwapPayload(
    btc: string,
): Promise<CreateSwapPayload> {
    const { make_create_sell_swap_payload } = await import("./wallet");
    return make_create_sell_swap_payload(WALLET_NAME, btc);
}
