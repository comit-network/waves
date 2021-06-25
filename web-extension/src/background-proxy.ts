import Debug from "debug";
import {
    Address,
    BalanceEntry,
    BalanceUpdate,
    BTC_TICKER,
    LoanToSign,
    Status,
    SwapToSign,
    USDT_TICKER,
    WalletStatus,
} from "./models";

const debug = Debug("bgproxy");

let walletStatus: WalletStatus = {
    status: Status.None,
};

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

export async function getAddress(): Promise<Address> {
    return Promise.resolve(
        "el1qqvrd63rn942zrr900nvnd4z37zhzdgtta3fpfzqmrcerht3wjllz0wccjhlrqtl2c8w6aggkek2pwvgcwhf5y2nwzjccu9avz",
    );
}

export async function signAndSend(tx: string): Promise<string> {
    return Promise.resolve("8ec2ff513cb55b621af73130818c359aef357038905b7954775eff43e92916f9");
}

export async function getLoanToSign(): Promise<LoanToSign | undefined> {
    let ran = Math.random();
    if (ran < 0.5) {
        return Promise.resolve(undefined);
    }

    let loanToSign: LoanToSign = {
        collateral: {
            ticker: BTC_TICKER,
            amount: 1,
            balanceBefore: 1,
            balanceAfter: 0,
        },
        principal: {
            ticker: USDT_TICKER,
            amount: 100000,
            balanceBefore: 0,
            balanceAfter: 100000,
        },
        principalRepayment: 110000,
        tabId: 0,
        term: 0,
        txHex: "0x00",
    };
    return Promise.resolve(loanToSign);
}
export async function getSwapToSign(): Promise<SwapToSign | undefined> {
    let ran = Math.random();
    if (ran < 0.5) {
        return Promise.resolve(undefined);
    }

    let swapToSign: SwapToSign = {
        decoded: {
            buy: {
                ticker: USDT_TICKER,
                amount: 100000,
                balanceBefore: 0,
                balanceAfter: 100000,
            },
            sell: {
                ticker: BTC_TICKER,
                amount: 1,
                balanceBefore: 1,
                balanceAfter: 0,
            },
        },
        tabId: 0,
        txHex: "0x00",
    };
    return Promise.resolve(swapToSign);
}

export async function cancelLoan(_loanToSign: LoanToSign): Promise<void> {
    return Promise.resolve();
}

export async function cancelSwap(_swapToSign: SwapToSign): Promise<void> {
    return Promise.resolve();
}

export async function withdrawAll(_address: string): Promise<void> {
    return Promise.resolve();
}
