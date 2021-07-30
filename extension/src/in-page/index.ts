import Debug from "debug";
import { Direction, Message, MessageKind } from "../messages";
import { Address, CreateSwapPayload, LoanRequestPayload, Tx, Txid, WalletStatus } from "../models";

Debug.enable("*");
const debug = Debug("inpage");

export default class WavesProvider {
    public async walletStatus(): Promise<WalletStatus> {
        debug("Requesting wallet status");
        let promise = new Promise<WalletStatus>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<WalletStatus>>) {
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.WalletStatusResponse
                ) {
                    if (event.data.error) {
                        reject(event.data.error);
                    } else {
                        debug(`Received wallet status: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        resolve(event.data.payload);
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.WalletStatusRequest,
            direction: Direction.ToBackground,
        }, "*");
        return promise;
    }

    public async getSellCreateSwapPayload(btc: string): Promise<CreateSwapPayload> {
        debug("Getting sell create-swap payload");
        let promise = new Promise<CreateSwapPayload>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<CreateSwapPayload>>) {
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.SellResponse
                ) {
                    if (event.data.error) {
                        reject(event.data.error);
                    } else {
                        debug(`Received sell response: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        resolve(event.data.payload);
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.SellRequest,
            direction: Direction.ToBackground,
            payload: btc,
        }, "*");
        return promise;
    }

    public async getBuyCreateSwapPayload(usdt: string): Promise<CreateSwapPayload> {
        debug("Getting buy create-swap payload");
        let promise = new Promise<CreateSwapPayload>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<CreateSwapPayload>>) {
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.BuyResponse
                ) {
                    if (event.data.error) {
                        reject(event.data.error);
                    } else {
                        debug(`Received buy response: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        resolve(event.data.payload);
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.BuyRequest,
            direction: Direction.ToBackground,
            payload: usdt,
        }, "*");
        return promise;
    }

    public async getNewAddress(): Promise<Address> {
        debug("Getting address");
        let promise = new Promise<Address>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<Address>>) {
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.AddressResponse
                ) {
                    if (event.data.error) {
                        reject(event.data.kind);
                    } else {
                        debug(`Received address: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        resolve(event.data.payload);
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.AddressRequest,
            direction: Direction.ToBackground,
        }, "*");
        return promise;
    }

    public async makeLoanRequestPayload(
        collateral: string,
        fee_rate: string,
        timeout: string,
    ): Promise<LoanRequestPayload> {
        debug("Making loan request payload");
        let promise = new Promise<LoanRequestPayload>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<LoanRequestPayload>>) {
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.LoanResponse
                ) {
                    if (event.data.error) {
                        reject(event.data.error);
                    } else {
                        debug(`Received loan request payload: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        resolve(event.data.payload);
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.LoanRequest,
            direction: Direction.ToBackground,
            payload: {
                collateral: collateral,
                fee_rate: fee_rate,
                timeout: timeout,
            },
        }, "*");
        return promise;
    }

    public async signAndSendSwap(tx_hex: string): Promise<Txid> {
        debug("Signing and sending swap");
        let promise = new Promise<Txid>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<Txid>>) {
                if (
                    event.data.direction === Direction.ToPage
                ) {
                    if (event.data.kind === MessageKind.SwapTxid) {
                        if (event.data.error) {
                            reject(event.data.error);
                        } else {
                            debug(`Received swap txid: ${JSON.stringify(event.data)}`);

                            window.removeEventListener("message", listener);
                            resolve(event.data.payload);
                        }
                    } else if (event.data.kind === MessageKind.SwapRejected) {
                        debug(`Swap rejected: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        reject("User rejected swap");
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.SignAndSendSwap,
            direction: Direction.ToBackground,
            payload: tx_hex,
        }, "*");
        return promise;
    }

    public async signLoan(loan_response: any): Promise<Tx> {
        debug("Signing loan after user confirmation");
        let promise = new Promise<Tx>((resolve, reject) => {
            let listener = async function(event: MessageEvent<Message<Tx>>) {
                if (
                    event.data.direction === Direction.ToPage
                ) {
                    if (event.data.kind === MessageKind.SignedLoan) {
                        if (event.data.error) {
                            reject(event.data.error);
                        } else {
                            debug(`Received signed loan: ${JSON.stringify(event.data)}`);

                            window.removeEventListener("message", listener);
                            resolve(event.data.payload);
                        }
                    } else if (event.data.kind === MessageKind.LoanRejected) {
                        debug(`Loan rejected: ${JSON.stringify(event.data)}`);

                        window.removeEventListener("message", listener);
                        reject("User rejected loan");
                    }
                }
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            kind: MessageKind.SignLoan,
            direction: Direction.ToBackground,
            payload: loan_response,
        }, "*");
        return promise;
    }
}

const initializeProvider = () => {
    debug("I was injected 🥳");
    // @ts-ignore `provider` is not known on `window`. That's why we are defining it ;)
    window.wavesProvider = new WavesProvider();
};

initializeProvider();
