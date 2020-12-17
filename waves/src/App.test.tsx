import { render, screen } from "@testing-library/react";
import React from "react";
import App, { AssetType, reducer } from "./App";

test("renders create new wallet", () => {
    render(<App />);
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
    rate: 0,
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
        rate: 10,
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
        rate: 19133.74,
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
        rate: 19133.74,
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
        rate: 19133.74,
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

test("Rate change - should update the amounts accordingly", () => {
    const initialState = {
        ...defaultState,
        alpha: {
            type: AssetType.BTC,
            amount: 10,
        },
        beta: {
            type: AssetType.USDT,
            amount: 1000,
        },
        rate: 100,
    };

    let newRate = 110;
    let newAmount = 1100;
    expect(
        reducer(initialState, {
            type: "UpdateRate",
            value: newRate,
        }).beta.amount,
    ).toBe(newAmount);

    // alpha amount should remain unchanged
    expect(
        reducer(initialState, {
            type: "UpdateRate",
            value: newRate,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);
});
