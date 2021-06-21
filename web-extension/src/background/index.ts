import Debug from "debug";
import { helloWorld } from "../wasmProxy";

Debug.enable("*");
const debug = Debug("background");

messageInBackground();

// This needs to be an export due to typescript implementation limitation of needing '--isolatedModules' tsconfig
export function messageInBackground() {
    debug("I can run your javascript like any other code in your project");
    debug("just do not forget, I cannot render anything !");
}

helloWorld();
