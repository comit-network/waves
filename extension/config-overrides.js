const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const webpack = require("webpack");

module.exports = function override(config, env) {
    config.entry.in_page = path.join(__dirname, "src", "in-page", "index.ts");

    config.resolve.extensions.push(".wasm");

    config.module.rules.forEach(rule => {
        if (!rule.oneOf) {
            return;
        }

        let fileLoader = rule.oneOf[rule.oneOf.length - 1];

        fileLoader.exclude.push(/\.wasm$/);
    });

    config.plugins = (config.plugins || []).concat([
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "wallet/"),
            outDir: path.resolve(__dirname, "src/wallet"),
        }),
        // delete the warning about "Critical dependency: the request of a dependency is an expression" in the generated binding code
        new webpack.ContextReplacementPlugin(
            /wallet/,
            (data) => {
                delete data.dependencies[0].critical;

                return data;
            },
        ),
    ]);

    config.experiments = {
        syncWebAssembly: true,
        asyncWebAssembly: true,
    };

    return config;
};
