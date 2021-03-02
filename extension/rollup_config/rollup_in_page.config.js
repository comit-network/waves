import rust from "@wasm-tool/rollup-plugin-rust";

export default {
    input: {
        in_page: "in_page/Cargo.toml",
    },
    output: {
        dir: "dist",
        format: "esm",
        entryFileNames: "js/[name].js",
    },
    plugins: [
        rust({
            inlineWasm: true,
        }),
    ],
};
