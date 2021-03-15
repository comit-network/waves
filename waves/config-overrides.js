const FileManagerPlugin = require("filemanager-webpack-plugin");

module.exports = function override(config, env) {
    config.plugins = (config.plugins || []).concat([
        new FileManagerPlugin({
            events: {
                onStart: {
                    delete: ["./dist/[!.gitignore]*"],
                },
                onEnd: {
                    copy: [
                        { source: "./build", destination: "./dist" },
                    ],
                },
            },
        }),
    ]);

    return config;
};
