import rust from "@wasm-tool/rollup-plugin-rust";

export default {
    input: {
        content: "content/Cargo.toml",
    },
    output: {
        dir: "dist",
        format: "iife",
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
