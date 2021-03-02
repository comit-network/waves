import rust from "@wasm-tool/rollup-plugin-rust";

export default {
    input: {
        background: "background/Cargo.toml",
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
    ],
};
