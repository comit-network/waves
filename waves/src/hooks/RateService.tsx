import { createContext, useContext } from "react";

export const RateServiceContext = createContext<string>("https://getbestrate.com");

export const Provider = RateServiceContext.Provider;

export const useRateService = () => {
    const initUrl = useContext(RateServiceContext);
    return new RateService(initUrl);
};

class RateService {
    private rate: number;
    constructor(private _url: string) {
        this.rate = 19_113.03;
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

export default RateService;