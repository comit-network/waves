import { render, screen } from "@testing-library/react";
import React, { useReducer } from "react";
import App, { AssetType } from "./App";
import { reducer } from "./App";

test("renders unlock wallet", () => {
    render(<App />);
    const linkElement = screen.getByText(/Unlock Wallet/i);
    expect(linkElement).toBeInTheDocument();
});

test("update alpha amount logic", () => {
    const initialState = {
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
            type: "AlphaAmount",
            value: newValue,
        }).alpha.amount,
    ).toBe(newValue);

    let expectedNewAmount = 420;
    expect(
        reducer(initialState, {
            type: "AlphaAmount",
            value: newValue,
        }).beta.amount,
    ).toBe(expectedNewAmount);
});

test("update alpha asset logic - should flip asset types", () => {
    const initialState = {
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
            type: "AlphaAssetType",
            value: newValue,
        }).alpha.type,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "AlphaAssetType",
            value: newValue,
        }).beta.type,
    ).toBe(initialState.alpha.type);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "AlphaAssetType",
            value: newValue,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);

    expect(
        reducer(initialState, {
            type: "AlphaAssetType",
            value: newValue,
        }).beta.amount,
    ).toBe(initialState.beta.amount);
});

test("update beta asset logic - should flip asset types", () => {
    const initialState = {
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
            type: "BetaAssetType",
            value: newValue,
        }).beta.type,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "BetaAssetType",
            value: newValue,
        }).alpha.type,
    ).toBe(initialState.beta.type);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "BetaAssetType",
            value: newValue,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);

    expect(
        reducer(initialState, {
            type: "BetaAssetType",
            value: newValue,
        }).beta.amount,
    ).toBe(initialState.beta.amount);
});

test("Swap asset types", () => {
    const initialState = {
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
            type: "RateChange",
            value: newRate,
        }).beta.amount,
    ).toBe(newAmount);

    // alpha amount should remain unchanged
    expect(
        reducer(initialState, {
            type: "RateChange",
            value: newRate,
        }).alpha.amount,
    ).toBe(initialState.alpha.amount);
});
