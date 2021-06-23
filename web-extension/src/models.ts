enum Status {
    None,
    Loaded,
    NotLoaded,
}

export type Address = string;

export type WalletStatus = {
    status: Status;
    address?: Address;
};

export type BalanceEntry = {
    assetId: string;
    ticker: string;
    value: number;
};

export type BalanceUpdate = {
    balances: BalanceEntry[];
};

export type TradeSide = {
    ticker: string;
    amount: number;
    balanceBefore: number;
    balanceAfter: number;
};

export type Trade = {
    buy: TradeSide;
    sell: TradeSide;
};

export type SwapToSign = {
    txHex: string;
    decoded: Trade;
    tabId: number;
};

export type LoanToSign = {
    collateral: TradeSide;
    principal: TradeSide;
    principalRepayment: number;
    term: number;
    tabId: number;
};

export type SwapsToSign = {
    swaps: SwapToSign[];
};
