const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const Dotenv = require("dotenv-webpack")
const CopyWebPackPlugin = require("copy-webpack-plugin");

module.exports = {
  mode: "development",
  entry: {
    editor: path.resolve(__dirname, "editor.js"),
    index: path.resolve(__dirname, "index.js"),
  },
  output: {
    path: path.resolve(__dirname, "..", "dist/web"),
    filename: "[name].js",
  },
  plugins: [
    new Dotenv({
      path: path.resolve(__dirname, "..", ".env")
    }),
    new CopyWebPackPlugin({
        patterns: [
            {from: path.resolve(__dirname, "editor.html"), to: path.resolve(__dirname, "..", "dist", "web")},
            {from: path.resolve(__dirname, "index.html"), to: path.resolve(__dirname, "..", "dist", "web")},
        ]
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname),

      // Check https://rustwasm.github.io/wasm-pack/book/commands/build.html for
      // the available set of arguments.
      //
      // Optional space delimited arguments to appear before the wasm-pack
      // command. Default arguments are `--verbose`.
      args: "--log-level warn",
      // Default arguments are `--typescript --target browser --mode normal`.
      extraArgs: "--no-typescript --target web",

      // Optional array of absolute paths to directories, changes to which
      // will trigger the build.
      // watchDirectories: [],

      // The same asfilename the `--out-dir` option for `wasm-pack`
      outDir: path.resolve(__dirname, "pkg"),

      // If defined, `forceWatch` will force activate/deactivate watch mode for
      // `.rs` files.
      //
      // The default (not set) aligns watch mode for `.rs` files to Webpack's
      // watch mode.
      // forceWatch: true,

      // If defined, `forceMode` will force the compilation mode for `wasm-pack`
      //
      // Possible values are `development` and `production`.
      //
      // the mode `development` makes `wasm-pack` build in `debug` mode.
      // the mode `production` makes `wasm-pack` build in `release` mode.
      // forceMode: "development",

      // Controls plugin output verbosity, either 'info' or 'error'.
      // Defaults to 'info'.
      // pluginLogLevel: 'info'
    }),
  ],
  experiments: {
    syncWebAssembly: true,
  },
  devServer: {
    static: {
      directory: path.join(__dirname, "dist", "web"),
    },
    compress: true,
    port: 9000,
  },
};
