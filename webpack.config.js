var autoprefixer       = require('autoprefixer');
var ExtractTextPlugin  = require('extract-text-webpack-plugin');
var HtmlWebpackPlugin  = require('html-webpack-plugin');
var ManifestPlugin     = require('webpack-manifest-plugin');
var path               = require('path');
var webpack            = require('webpack');
var merge              = require('webpack-merge');

console.log('WEBPACK GO!');

// detemine build env
var TARGET_ENV = process.env.npm_lifecycle_event === 'build' ? 'prod' : 'dev';

// common webpack config
var assetsDir = path.resolve(__dirname, 'src/web');
var publicDir = path.resolve(__dirname, 'public');

var commonConfig = {

  output: {
    path: publicDir,
    filename: '[hash].js',
  },

  resolve: {
    modulesDirectories: ['node_modules'],
    extensions: ['', '.js'],
    root: [assetsDir, path.resolve(assetsDir, 'scripts')]
  },

  module: {
    loaders: [
     {
        test: /\.jsx?$/,
        exclude: /(node_modules|bower_components)/,
        loader: 'babel',
        query: {
          cacheDirectory: true
        }
      }, {
        test: /\.(woff2?|ttf|eot|svg)$/,
        loader: 'file?name=css/font/[name].[ext]'
      }, {
        test: /\.(jpg|png)$/,
        loader: 'file?name=img/[name].[ext]'
      }, {
          test: /\.(jpg|png)$/,
          loader: 'file?name=img/[name].[ext]'
      }
    ],
    preLoaders: [
      {test: /\.js$/, loader: 'eslint-loader', exclude: /node_modules/}
    ]
  },

  eslint: {
    failOnError: true
  },

  plugins: [
    // new webpack.ProvidePlugin({
    //   jQuery: 'jquery'
    // }),
    new HtmlWebpackPlugin({
      template: 'src/web/index.html',
      inject:   'body',
      filename: 'index.html'
    }),
    new ManifestPlugin()
  ],

  postcss: [autoprefixer({browsers: ['last 2 versions']})],

}

// additional webpack settings for local env (when invoked by 'npm start')
if (TARGET_ENV === 'dev') {
  console.log('Serving locally...');

  module.exports = merge(commonConfig, {

    devtool: '#source-map',

    entry: [
      'webpack-dev-server/client?http://localhost:8080',
      path.join( __dirname, 'src/web/scripts/index.js' )
    ],

    devServer: {
      inline: true,
      progress: true,
      headers: { "Access-Control-Allow-Origin": "*" }
    },

    module: {
      loaders: [
        {
          test: /\.css$/,
          loaders: [
            'style-loader',
            'css-loader',
            'postcss-loader?sourceMap'
          ]
        }, {
          test: /\.scss$/,
          loaders: [
            'style-loader?sourceMap',
            'css-loader?sourceMap',
            'postcss-loader?sourceMap',
            'sass-loader?sourceMap'
          ]
        }, {
          test: /\.less/,
          loaders: [
            'style-loader?sourceMap',
            'css-loader?sourceMap',
            'postcss-loader?sourceMap',
            'less-loader?sourceMap'
          ]
        }
      ]
    }

  });
}

// additional webpack settings for prod env (when invoked via 'npm run build')
if (TARGET_ENV === 'prod') {
  console.log( 'Building for prod...');

  module.exports = merge(commonConfig, {

    entry: path.join( __dirname, 'src/web/scripts/index.js' ),

    module: {
      loaders: [
        {
          test: /\.css$/,
          loader: ExtractTextPlugin.extract('style-loader', [
            'css-loader',
            'postcss-loader'
          ])
        }, {
          test: /\.scss$/,
          loader: ExtractTextPlugin.extract('style-loader', [
            'css-loader',
            'postcss-loader',
            'sass-loader'
          ])
        }, {
          test: /\.less$/,
          loader: ExtractTextPlugin.extract('style-loader', [
            'css-loader',
            'postcss-loader',
            'less-loader'
          ])
        }
      ]
    },

    plugins: [
      new webpack.optimize.OccurenceOrderPlugin(),

      // extract CSS into a separate file
      new ExtractTextPlugin('./css/[hash].css', {allChunks: true}),

      // minify & mangle JS/CSS
      new webpack.optimize.UglifyJsPlugin({
          minimize: true,
          compressor: {warnings: false}
          // mangle:  true
      })
    ]

  });
}
