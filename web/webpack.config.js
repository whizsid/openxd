const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const Dotenv = require("dotenv-webpack")

module.exports = {
  mode: "development",
  entry: path.resolve(__dirname, "index.js"),
  output: {
    path: path.resolve(__dirname, "..", "dist/web"),
    filename: "bundle.js",
  },
    plugins: [
      new Dotenv({
        path: path.resolve(__dirname,"..", ".env")
      }),
      new HtmlWebpackPlugin({
        templateContent: ({htmlWebpackPlugin}) => `
            <html>
              <head>
                <style type="text/css">
                    html {
                        /* Remove touch delay: */
                        touch-action: manipulation;
                    }
                    body {
                        /* Light mode background color for what is not covered by the egui canvas,
                        or where the egui canvas is translucent. */
                        background: #909090;
                    }
                    @media (prefers-color-scheme: dark) {
                        body {
                            /* Dark mode background color for what is not covered by the egui canvas,
                            or where the egui canvas is translucent. */
                            background: #404040;
                        }
                    }
                    /* Allow canvas to fill entire web page: */
                    html,
                    body {
                        overflow: hidden;
                        margin: 0 !important;
                        padding: 0 !important;
                        height: 100%;
                        width: 100%;
                    }
                    /* Position canvas in center-top: */
                    canvas {
                        margin-right: auto;
                        margin-left: auto;
                        display: block;
                        position: absolute;
                        top: 0%;
                        left: 50%;
                        transform: translate(-50%, 0%);
                    }
                    .centered {
                        margin-right: auto;
                        margin-left: auto;
                        display: block;
                        position: absolute;
                        top: 50%;
                        left: 50%;
                        transform: translate(-50%, -50%);
                        color: #f0f0f0;
                        font-size: 24px;
                        font-family: Ubuntu-Light, Helvetica, sans-serif;
                        text-align: center;
                    }
                    /* ---------------------------------------------- */
                    /* Loading animation from https://loading.io/css/ */
                    .lds-dual-ring {
                        display: inline-block;
                        width: 24px;
                        height: 24px;
                    }
                    .lds-dual-ring:after {
                        content: " ";
                        display: block;
                        width: 24px;
                        height: 24px;
                        margin: 0px;
                        border-radius: 50%;
                        border: 3px solid #fff;
                        border-color: #fff transparent #fff transparent;
                        animation: lds-dual-ring 1.2s linear infinite;
                    }
                    @keyframes lds-dual-ring {
                        0% {
                            transform: rotate(0deg);
                        }
                        100% {
                            transform: rotate(360deg);
                        }
                    }
                </style>
                ${htmlWebpackPlugin.tags.headTags}
              </head>
              <body>
                <canvas id="gui"></canvas>
                ${htmlWebpackPlugin.tags.bodyTags}
              </body>
            </html>
          `
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

      // The same as the `--out-dir` option for `wasm-pack`
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
