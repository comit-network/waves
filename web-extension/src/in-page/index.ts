import Debug from "debug";
import { Direction, Message, MessageKind } from "../messages";
import { CreateSwapPayload, WalletStatus } from "../models";

Debug.enable("inpage");
const debug = Debug("inpage");

export default class WavesProvider {
    public async walletStatus(): Promise<WalletStatus> {
        debug("Requesting wallet status");
        let promise = new Promise<WalletStatus>((resolve, _reject) => {
            let listener = async function(event: MessageEvent<Message<WalletStatus>>) {
                // TODO timeout and reject promise after some time of no response.
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.WalletStatusResponse
                ) {
                    debug(`Received wallet status: ${JSON.stringify(event.data)}`);

                    window.removeEventListener("message", listener);
                    resolve(event.data.payload);
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
        let promise = new Promise<CreateSwapPayload>((resolve, _reject) => {
            let listener = async function(event: MessageEvent<Message<CreateSwapPayload>>) {
                // TODO timeout and reject promise after some time of no response.
                if (
                    event.data.direction === Direction.ToPage
                    && event.data.kind === MessageKind.SellResponse
                ) {
                    debug(`Received sell response: ${JSON.stringify(event.data)}`);

                    window.removeEventListener("message", listener);
                    resolve(event.data.payload);
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
}

const initializeProvider = () => {
    debug("I was injected 🥳");
    // @ts-ignore `provider` is not known on `window`. That's why we are defining it ;)
    window.wavesProvider = new WavesProvider();
};

initializeProvider();
