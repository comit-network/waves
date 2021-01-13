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
export default function calculateBetaAmount(alphaAsset: Asset, amount: number, rate: Rate) {
    const direction = getDirection(alphaAsset);
    switch (direction) {
        case "bid":
            return amount * rate.bid;
        case "ask":
            return amount * (1 / rate.ask); // to make it obvious
    }
}

export function getDirection(alphaAsset: Asset) {
    // we only support these two assets right now
    switch (alphaAsset) {
        case Asset.LBTC:
            return "bid";
        case Asset.USDT:
            return "ask";
        default:
            // We default to ask rate. This is fine since it is only for the UI.
            return "ask";
    }
}
