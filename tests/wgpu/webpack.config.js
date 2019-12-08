const path = require('path');

module.exports = {
  entry: './out.js',
  output: {
    path: path.resolve(__dirname, '/test/wgpu/dist'),
    filename: 'bundle.js'
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: require.resolve('@open-wc/webpack-import-meta-loader'),
      },
    ],
  },
};