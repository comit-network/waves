export interface BalanceEntry {
    asset: string;
    value: bigint;
}

export interface WalletStatus {
    unlocked: boolean;
}

const WALLET_NAME = "wallet-1";

export async function getAddress() {
    const { get_address } = await import("./wallet/pkg");
    return get_address(WALLET_NAME);
}

export async function newWallet(password: string): Promise<WalletStatus> {
    const { create_new_wallet } = await import("./wallet/pkg");
    return create_new_wallet(WALLET_NAME, password).then(getWalletStatus);
}

export async function unlockWallet(password: string) {
    const { load_existing_wallet } = await import("./wallet/pkg");
    return load_existing_wallet(WALLET_NAME, password).then(getWalletStatus);
}

export async function lockWallet() {
    const { unload_current_wallet } = await import("./wallet/pkg");
    return unload_current_wallet().then(getWalletStatus);
}

export async function getWalletStatus(): Promise<WalletStatus> {
    const { wallet_status } = await import("./wallet/pkg");
    return wallet_status(WALLET_NAME);
}
