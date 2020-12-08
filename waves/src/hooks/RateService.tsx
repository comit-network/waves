import { createContext, useContext } from "react";

export const RateServiceContext = createContext<number>(20000);

export const Provider = RateServiceContext.Provider;

export const useRateService = () => {
    const initRate = useContext(RateServiceContext);
    return new RateService(initRate);
};

export default class RateService {
    constructor(private rate: number) {
    }

    public subscribe(callback: (rate: number) => void) {
        // just a dumdum
        return setTimeout(() => {
            this.rate += 10;
            callback(this.rate);
        }, 10_000);
    }

    unsubscribe(subscription: NodeJS.Timeout) {
        return clearTimeout(subscription);
    }
}
