import { AssetType, reducer } from "./App";
import { calculateBetaAmount } from "./RateService";

test("test bid amount", () => {
    const rate = {
        bid: 1900,
        ask: 2000,
    };
    const alpha = AssetType.BTC;

    const alphaAmount = 1.0;
    const expectedBetaAmount = 1900;

    const betaAmount = calculateBetaAmount(alpha, alphaAmount, rate);

    expect(betaAmount).toBe(expectedBetaAmount);
});

test("test ask amount", () => {
    const rate = {
        bid: 1900,
        ask: 2000,
    };
    const alpha = AssetType.USDT;

    const alphaAmount = 2000;
    const expectedBetaAmount = 1.0;

    const betaAmount = calculateBetaAmount(alpha, alphaAmount, rate);

    expect(betaAmount).toBe(expectedBetaAmount);
});
