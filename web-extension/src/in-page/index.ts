import Debug from "debug";

Debug.enable("inpage");
const debug = Debug("inpage");

export default class WavesProvider {
    public async callBackend(): Promise<string> {
        debug("sending message...");

        let promise = new Promise<string>((resolve, _reject) => {
            let listener = async function(event: MessageEvent) {
                // TODO timeout and reject promise after some time of no response.
                if (
                    event.data
                    && event.data.direction !== "to-page"
                ) {
                    // ignored
                    return;
                }
                debug(`Response: "${event.data.message}"`);
                // TODO: check if this was the response we were waiting for, if not, do not yet deregister this listener
                window.removeEventListener("message", listener);
                resolve(event.data.message);
            };
            window.addEventListener("message", listener);
        });
        window.postMessage({
            direction: "to-background",
            message: "Message from the page",
        }, "*");
        return promise;
    }
}

const initialize_provider = () => {
    debug("I was injected 🥳");
    // @ts-ignore `provider` is not known on `window`. That's why we are defining it ;)
    window.wavesProvider = new WavesProvider();
};

initialize_provider();
