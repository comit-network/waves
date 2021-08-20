import debug from "debug";
import { AsyncReturnType } from "type-fest";
import { v4 } from "uuid";
import { CreateSwapPayload, LoanRequestPayload, Wallet, WalletStatus } from "../background/api";
import { RpcRequest, RpcResponse } from "../contentScript";
import { ParametersObject } from "../type-utils";

const log = debug("inpage");

export class InvokeEventListenerViaContentScript implements Wallet {
    walletStatus(): Promise<WalletStatus> {
        return invokeContentScript("walletStatus", {});
    }
    getNewAddress(): Promise<string> {
        return invokeContentScript("getNewAddress", {});
    }
    makeSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload> {
        return invokeContentScript("makeSellCreateSwapPayload", { btc });
    }
    makeBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload> {
        return invokeContentScript("makeBuyCreateSwapPayload", { usdt });
    }
    makeLoanRequestPayload(collateral: string, fee_rate: string): Promise<LoanRequestPayload> {
        return invokeContentScript("makeLoanRequestPayload", { collateral, fee_rate });
    }
    requestSignSwap(hex: string): Promise<string> {
        return invokeContentScript("requestSignSwap", { hex });
    }
    requestSignLoan(loanRequest: LoanRequestPayload): Promise<string> {
        return invokeContentScript("requestSignLoan", { loanRequest });
    }
}

function invokeContentScript<R extends keyof Wallet>(
    fn: R,
    args: ParametersObject<Wallet[R]>,
): Promise<AsyncReturnType<Wallet[R]>> {
    let request = new Request(fn, args);

    log(`Sending request for %s with %s`, fn, request.id);

    let promise = new Promise<AsyncReturnType<Wallet[R]>>((resolve, reject) => {
        let listener = function(event: MessageEvent) {
            if (!request.isRespondedToIn(event)) {
                return;
            }

            log(`Received response for %s`, fn);

            if (event.data.err) {
                reject(event.data.err);
            } else if (event.data.ok) {
                resolve(event.data.ok);
            } else {
                throw new Error("Invalid event format!");
            }

            window.removeEventListener("message", listener);
        };

        window.addEventListener("message", listener);
    });

    request.send();

    return promise;
}

class Request<T extends keyof Wallet> {
    readonly id: string;

    constructor(readonly method: T, readonly args: ParametersObject<Wallet[T]>) {
        this.id = v4();
    }

    /**
     * Checks if the given event is the response to this request.
     *
     * This uses TypeScript's Type Guards: https://www.typescripttutorial.net/typescript-tutorial/typescript-type-guards/
     */
    isRespondedToIn(event: MessageEvent): event is MessageEvent<RpcResponse<T>> {
        return event.data.type === "wallet-rpc-response" && event.data.id === this.id;
    }

    send() {
        let message: RpcRequest<T> = {
            type: "wallet-rpc-request",
            method: this.method,
            args: this.args,
            id: this.id,
        };

        window.postMessage(message, "*");
    }
}
