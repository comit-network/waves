import Debug from "debug";
Debug.enable("inpage");
const debug = Debug("inpage");

export default class WavesProvider {
    public async callBackend() {
        debug("sending message...");
        window.postMessage({
            direction: "from-page-script",
            message: "Message from the page",
        }, "*");
    }
}

const initialize_provider = () => {
    debug("I was injected 🥳");
    // @ts-ignore `provider` is not known here but we create it on `window` ;)
    window.provider = new WavesProvider();
};

initialize_provider();
