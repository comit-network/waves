import { useAsync } from "react-async";
import { backgroundPage, BalanceEntry, LoanDetails, LoanToSign, Trade, WalletStatus } from "./background/api";

export function useWalletStatus() {
    return useAsync({ promiseFn: getWalletStatus });
}

export function useBalances() {
    return useAsync({ promiseFn: getBalances });
}

export function useSwapToSign() {
    return useAsync({ promiseFn: getSwapToSign });
}

export function useLoanToSign() {
    return useAsync({ promiseFn: getLoanToSign });
}

export function useOpenLoans() {
    return useAsync({ promiseFn: getOpenLoans });
}

export function useAddress() {
    return useAsync({ promiseFn: getAddress });
}

// The useAsync hook requires us to define this functions so we can reference them.
// Using `useAsync` with an inline arrow function would repeatedly cancel the promise and create a new one.
// See the docs of `useAsync` for more details: https://docs.react-async.com/api/options#promisefn

async function getWalletStatus(): Promise<WalletStatus> {
    const page = await backgroundPage();

    return page.getWalletStatus();
}

async function getBalances(): Promise<BalanceEntry[]> {
    const page = await backgroundPage();

    return page.getBalances();
}

async function getSwapToSign(): Promise<Trade | null> {
    const page = await backgroundPage();

    if (!page.swapToSign) {
        return Promise.resolve(null);
    }

    return page.swapToSign.decoded;
}

async function getLoanToSign(): Promise<LoanToSign | null> {
    const page = await backgroundPage();

    return page.loanToSign;
}

async function getOpenLoans(): Promise<LoanDetails[]> {
    const page = await backgroundPage();

    return page.getOpenLoans();
}

async function getAddress(): Promise<string> {
    const page = await backgroundPage();

    return page.getAddress();
}
