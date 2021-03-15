const CopyWebpackPlugin = require("copy-webpack-plugin");
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  module: {
    rules: [{
      test: /\.css$/,
      use: ['style-loader', 'css-loader']
    }, {
      test: /\.ttf$/,
      use: ['file-loader']
    }]
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html']),
    new MonacoWebpackPlugin({
      languages: ["json"]
    })
  ],
};
