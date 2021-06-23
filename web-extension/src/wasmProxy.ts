import Debug from "debug";

Debug.enable("*");
let debug = Debug("wasm-proxy");

export async function helloWorld() {
    debug("Loading wasm lib");
    const { hello_world } = await import("./wallet");
    hello_world();
}
