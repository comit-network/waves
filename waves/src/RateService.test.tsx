import { Asset } from "./App";
import { calculateBetaAmount } from "./RateService";

test("test bid amount", () => {
    const rate = {
        bid: 1900,
        ask: 2000,
    };
    const alpha = Asset.LBTC;

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
    const alpha = Asset.USDT;

    const alphaAmount = 2000;
    const expectedBetaAmount = 1.0;

    const betaAmount = calculateBetaAmount(alpha, alphaAmount, rate);

    expect(betaAmount).toBe(expectedBetaAmount);
});
