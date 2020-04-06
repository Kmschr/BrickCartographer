const path = require('path');
const webpack = require('webpack');

const dev_port = 31401;

module.exports = {
  mode: 'development',
  devServer: {
    port: dev_port,
    open: true,
    hot: true,
  },
  entry: [
    'babel-polyfill',
    './src/js/entry.jsx'
  ],
  output: {
    path: path.join(__dirname, '/'),
    filename: 'bundle.js',
  },
  plugins: [
    new webpack.NamedModulesPlugin(),
    new webpack.HotModuleReplacementPlugin()
  ],
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/, use: [{
          loader: 'babel-loader',
          options: {
            presets: ['react', 'env'],
            plugins: ['babel-plugin-syntax-dynamic-import']
          }
        }],
        include: path.join(__dirname, './', 'src/js'),
      },
      { test: /\.css$/, use: ['style-loader', 'css-loader'] },
      { test: /\.(png|jpg|gif|svg)$/, use: {
        loader: 'file-loader',
        options: { name: 'images/[name]-[hash:8].[ext]' }}
      }
    ]
  }
};
