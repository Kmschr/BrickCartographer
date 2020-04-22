const path = require('path');

module.exports = {
  mode: 'production',
  entry: [
    'babel-polyfill',
    './js/entry.jsx'
  ],
  output: {
    path: path.join(__dirname, './dist/public/'),
    filename: 'bundle.js',
  },
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
        include: path.join(__dirname, 'js'),
      },
      { test: /\.css$/, use: ['style-loader', 'css-loader'] },
      { test: /\.(png|jpg|gif|svg|brs)$/, use: {
        loader: 'file-loader',
        options: { name: '[path][name]-[hash:8].[ext]' } }
      }
    ]
  }
}
