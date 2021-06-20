import Debug from "debug";

Debug.enable("*");
let debug = Debug("wasm-proxy");

export async function helloWorld() {
    debug("Loading wasm lib");
    const { setup } = await import("./wallet");
    setup();
}
