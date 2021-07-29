import { act, render, screen } from "@testing-library/react";
import React from "react";
import { Listener, Source, SSEProvider } from "react-hooks-sse";
import { BrowserRouter } from "react-router-dom";
import App, { Asset, reducer } from "./App";
import { Interest, Rate } from "./Bobtimus";
import calculateBetaAmount from "./calculateBetaAmount";

const defaultLoanOffer = {
    rate: {
        ask: 20000,
        bid: 20000,
    },
    fee_sats_per_vbyte: 1,
    min_principal: 100,
    max_principal: 10000,
    max_ltv: 0.8,
    interest: [{
        timelock: 43200,
        interest_rate: 0.15,
    }],
};

const defaultState = {
    trade: {
        alpha: {
            type: Asset.LBTC,
            amount: "0",
        },
        beta: Asset.USDT,
        txId: "",
    },
    borrow: {
        principalAmount: "1000",
        loanTerm: 30,
        loanOffer: defaultLoanOffer,
    },
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
        trade: {
            ...defaultState.trade,
            alpha: {
                type: Asset.LBTC,
                amount: "0.01",
            },
            beta: Asset.USDT,
        },
    };

    let newValue = "42";
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAmount",
            value: newValue,
        }).trade.alpha.amount,
    ).toBe(newValue);
});

test("update alpha asset logic - should flip asset types", () => {
    const initialState = {
        ...defaultState,
        trade: {
            ...defaultState.trade,
            alpha: {
                type: Asset.LBTC,
                amount: "0.01",
            },
            beta: Asset.USDT,
        },
    };

    let newValue = Asset.USDT;
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).trade.alpha.type,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).trade.beta,
    ).toBe(initialState.trade.alpha.type);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "UpdateAlphaAssetType",
            value: newValue,
        }).trade.alpha.amount,
    ).toBe(initialState.trade.alpha.amount);
});

test("update beta asset logic - should flip asset types", () => {
    const initialState = {
        ...defaultState,
        trade: {
            ...defaultState.trade,
            alpha: {
                type: Asset.LBTC,
                amount: "0.01",
            },
            beta: Asset.USDT,
        },
    };

    let newValue = Asset.LBTC;
    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).trade.beta,
    ).toBe(newValue);

    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).trade.alpha.type,
    ).toBe(initialState.trade.beta);

    // amounts should be unchanged
    expect(
        reducer(initialState, {
            type: "UpdateBetaAssetType",
            value: newValue,
        }).trade.alpha.amount,
    ).toBe(initialState.trade.alpha.amount);
});

test("Swap asset types", () => {
    const initialState = {
        ...defaultState,
        trade: {
            ...defaultState.trade,
            alpha: {
                type: Asset.LBTC,
                amount: "0.01",
            },
            beta: Asset.USDT,
        },
    };

    const rate = {
        bid: 10,
        ask: 10,
    };
    // This is just showing how the beta amount is calculated in "reality". The actual amounts and rates don't matter
    // in this test.
    let amountAsNumber = Number.parseFloat(initialState.trade.alpha.amount);
    let betaAmount = calculateBetaAmount(initialState.trade.alpha.type, amountAsNumber, rate);

    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
            value: {
                betaAmount: betaAmount,
            },
        }).trade.alpha.type,
    ).toBe(initialState.trade.beta);

    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
            value: {
                betaAmount: betaAmount,
            },
        }).trade.beta,
    ).toBe(initialState.trade.alpha.type);

    // amounts should not be flipped.
    expect(
        reducer(initialState, {
            type: "SwapAssetTypes",
            value: {
                betaAmount: betaAmount,
            },
        }).trade.alpha.amount,
    ).toBe(
        initialState.trade.alpha.amount.toString(),
    );
});

test("update principal amount logic", () => {
    const initialState = {
        ...defaultState,
        borrow: {
            loanTerm: 30,
            principalAmount: "10000",
            loanOffer: null,
        },
    };

    let newValue = "42";
    expect(
        reducer(initialState, {
            type: "UpdatePrincipalAmount",
            value: newValue,
        }).borrow.principalAmount,
    ).toBe(newValue);
});

test("update loan offer logic", () => {
    const initialState = {
        ...defaultState,
        borrow: {
            loanTerm: 0,
            principalAmount: "0",
            loanOffer: null,
        },
    };

    let newState = reducer(initialState, {
        type: "UpdateLoanOffer",
        value: defaultLoanOffer,
    });

    expect(newState.borrow.loanOffer).toBe(defaultLoanOffer);
    expect(newState.borrow.principalAmount).toBe("100");
    expect(newState.borrow.loanTerm).toBe(30);
});
