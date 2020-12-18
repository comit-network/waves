import { render, screen } from "@testing-library/react";
import React from "react";
import { Listener, Source, SSEProvider } from "react-hooks-sse";
import App, { AssetType, reducer } from "./App";

// implementation of customSource does not matter but functions need to be there
class CustomSource implements Source {
    addEventListener(name: string, listener: Listener) {
    }

    close() {
    }

    removeEventListener(name: string, listener: Listener) {
    }
}

test("Test if rendering works by asserting `create new wallet` button", () => {
    render(
        <SSEProvider source={() => new CustomSource()}>
            <App />
        </SSEProvider>,
    );
    const linkElement = screen.getByText(/Create new wallet/i);
    expect(linkElement).toBeInTheDocument();
});

const defaultState = {
    alpha: {
        type: AssetType.BTC,
        amount: 0,
    },
    beta: {
        type: AssetType.USDT,
        amount: 0,
    },
    rate: {
        bid: 19000,
        ask: 20000,
    },
    txId: "",
    wallet: {
        balance: {
            usdtBalance: 0,
            btcBalance: 0,
        },
        status: {
            exists: false,
            loaded: false,
        },
    },
};

test("update alpha amount logic", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 0.01,
        },
        beta: {
            type: AssetType.USDT,
            amount: 191.34,
        },
        rate: {
            bid: 10,
            ask: 9,
        },
    };

    let newValue = 42;
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAmount",
            value: newValue,
        }).alpha.amount,
    ).toBe(newValue);

    let expectedNewAmount = 420;
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAmount",
            value: newValue,
        }).beta.amount,
    ).toBe(expectedNewAmount);
});

test("update alpha asset logic - should flip asset types", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 0.01,
        },
        beta: {
            type: AssetType.USDT,
            amount: 191.34,
        },
    };

    let newValue = AssetType.USDT;
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).alpha.type,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).beta.type,
    ).toBe(initialState.alpha.type);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);

    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).beta.amount,
    ).toBe(initialState.beta.amount);
});

test("update beta amount if rate changed", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 1,
        },
        beta: {
            type: AssetType.USDT,
            amount: 19000,
        },
        rate: {
            bid: 19000,
            ask: 20000,
        },
    };

    let newRate = {
        ask: 21000,
        bid: 20000,
    };

    expect(
        reducer(initialState, {
            type: "UpdateBetaAmount",
            value: newRate,
        }).beta.amount,
    ).toBe(20000);
});

test("update beta asset logic - should flip asset types", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 0.01,
        },
        beta: {
            type: AssetType.USDT,
            amount: 191.34,
        },
    };

    let newValue = AssetType.BTC;
    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).beta.type,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).alpha.type,
    ).toBe(initialState.beta.type);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);

    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).beta.amount,
    ).toBe(initialState.beta.amount);
});

test("Swap asset types", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 0.01,
        },
        beta: {
            type: AssetType.USDT,
            amount: 191.34,
        },
    };

    let newValue = AssetType.BTC;
    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
        }).alpha.type,
    ).toBe(initialState.beta.type);

    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
        }).beta.type,
    ).toBe(initialState.alpha.type);

    // amounts should be flipped as well
    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
        }).alpha.amount,
    ).toBe(initialState.beta.amount);

    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
            value: newValue,
        }).beta.amount,
    ).toBe(initialState.alpha.amount);
});
