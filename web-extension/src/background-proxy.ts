import Debug from "debug";
import { Address, BalanceEntry, BalanceUpdate, BTC_TICKER, Status, USDT_TICKER, WalletStatus } from "./models";

const debug = Debug("bgproxy");

let walletStatus: WalletStatus = {
    status: Status.None,
};

export async function getWalletStatus(): Promise<WalletStatus> {
    debug("Getting wallet status");
    return Promise.resolve(walletStatus);
}

export async function getWalletBalance(): Promise<BalanceUpdate> {
    debug("Getting wallet balance");
    if (walletStatus.status !== Status.Loaded) {
        return Promise.resolve({ balances: [] });
    }
    let usdt: BalanceEntry = {
        assetId: "x...USDT",
        ticker: USDT_TICKER,
        value: 10,
    };
    let btc: BalanceEntry = {
        assetId: "x...BTC",
        ticker: BTC_TICKER,
        value: 1,
    };
    return Promise.resolve({ balances: [usdt, btc] });
}

export async function unlockWallet(_password: string): Promise<void> {
    debug("Unlocking wallet");
    walletStatus = {
        status: Status.Loaded,
    };
    return Promise.resolve();
}
export async function createWallet(_password: string): Promise<void> {
    debug("Creating wallet");
    walletStatus = {
        status: Status.Loaded,
    };
    return Promise.resolve();
}

export async function getAddress(): Promise<Address> {
    return Promise.resolve(
        "el1qqvrd63rn942zrr900nvnd4z37zhzdgtta3fpfzqmrcerht3wjllz0wccjhlrqtl2c8w6aggkek2pwvgcwhf5y2nwzjccu9avz",
    );
}

export async function signAndSend(tx: string): Promise<string> {
    return Promise.resolve("8ec2ff513cb55b621af73130818c359aef357038905b7954775eff43e92916f9");
}
