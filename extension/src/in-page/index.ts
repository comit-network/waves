import Debug from "debug";
import { AsyncReturnType } from "type-fest";
import { v4 } from "uuid";
import { RequestMessage, ResponseMessage } from "../contentScript";
import { Address, CreateSwapPayload, LoanRequestPayload, Tx, Txid, WalletStatus } from "../models";

Debug.enable("*");
const debug = Debug("inpage");

export default class WavesProvider {
    public async walletStatus(): Promise<WalletStatus> {
        return invokeContentScript("walletStatus", []);
    }

    public async getSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload> {
        return invokeContentScript("getSellCreateSwapPayload", [btc]);
    }

    public async getBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload> {
        return invokeContentScript("getBuyCreateSwapPayload", [usdt]);
    }

    public async getNewAddress(): Promise<Address> {
        return invokeContentScript("getNewAddress", []);
    }

    public async makeLoanRequestPayload(
        collateral: string,
        fee_rate: string,
        timeout: string,
    ): Promise<LoanRequestPayload> {
        return invokeContentScript("makeLoanRequestPayload", [collateral, fee_rate, timeout]);
    }

    public async signAndSendSwap(tx_hex: string): Promise<Txid> {
        return invokeContentScript("signAndSendSwap", [tx_hex]);
    }

    public async signLoan(loan_response: any): Promise<Tx> {
        return invokeContentScript("signLoan", [loan_response]);
    }
}

function invokeContentScript<R extends keyof WavesProvider>(
    fn: R,
    args: Parameters<WavesProvider[R]>,
): Promise<AsyncReturnType<WavesProvider[R]>> {
    let request = new Request(fn, args);

    debug(`Sending request for %s with %s`, fn, request.id);

    let promise = new Promise<AsyncReturnType<WavesProvider[R]>>((resolve, reject) => {
        let listener = function(event: MessageEvent) {
            if (!request.isRespondedToIn(event)) {
                return;
            }

            debug(`Received response for %s`, fn);

            if (event.data.err) {
                reject(event.data.err);
            } else if (event.data.ok) {
                resolve(event.data.ok);
            } else {
                debug("Bad event! %s", JSON.stringify(event.data));

                throw new Error("Invalid event format!");
            }

            window.removeEventListener("message", listener);
        };

        window.addEventListener("message", listener);
    });

    request.send();

    return promise;
}

class Request<T extends keyof WavesProvider> {
    readonly id: string;

    constructor(readonly method: T, readonly args: Parameters<WavesProvider[T]>) {
        this.id = v4();
    }

    /**
     * Checks if the given event is the response to this request.
     *
     * This uses TypeScript's Type Guards: https://www.typescripttutorial.net/typescript-tutorial/typescript-type-guards/
     */
    isRespondedToIn(event: MessageEvent): event is MessageEvent<ResponseMessage<T>> {
        return event.data.type === "response" && event.data.id === this.id;
    }

    send() {
        let message: RequestMessage<T> = {
            type: "request",
            method: this.method,
            args: this.args,
            id: this.id,
        };

        window.postMessage(message, "*");
    }
}

const initializeProvider = () => {
    debug("I was injected 🥳");
    // @ts-ignore `provider` is not known on `window`. That's why we are defining it ;)
    window.wavesProvider = new WavesProvider();
};

initializeProvider();
