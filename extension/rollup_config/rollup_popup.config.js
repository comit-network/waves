import rust from "@wasm-tool/rollup-plugin-rust";
import copy from "rollup-plugin-copy";

export default {
    input: {
        popup: "Cargo.toml",
    },
    output: {
        dir: "dist",
        format: "esm",
        entryFileNames: "js/[name].js",
    },
    plugins: [
        rust({
            importHook: function(path) {
                return "browser.runtime.getURL(" + JSON.stringify(path) + ")";
            },
        }),
        copy({
            targets: [
                { src: "static/*", dest: "dist" },
            ],
        }),
    ],
};
