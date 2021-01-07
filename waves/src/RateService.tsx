import { Asset, Rate } from "./App";

/**
 * returns a (beta) amount based on alphaAsset, amount and rate.
 * Currently only BTC and USDT are accepted
 * @param alphaAsset needs to be either AssetType.BTC or AssetType.USDT
 * @param amount the alpha amount is used as the multiplicand
 * @param rate depicts a bid and ask rate which is used as the multiplier:
 *  The bid price is what the LP is willing to pay for the currency, while
 *  the ask price is the rate at which the LP will sell the same currency.
 */
export const calculateBetaAmount = (alphaAsset: Asset, amount: number, rate: Rate) => {
    type Direction = "ask" | "bid";
    let direction: Direction = "ask";
    // we only support these two assets right now
    switch (alphaAsset) {
        case Asset.LBTC:
            direction = "bid";
            break;
        case Asset.USDT:
            direction = "ask";
            break;
        default:
            // TODO error
            break;
    }

    switch (direction) {
        case "bid":
            return amount * rate.bid;
        case "ask":
            return amount * (1 / rate.ask); // to make it obvious
    }
};
