import Immutable from 'immutable';
import { createStore, applyMiddleware, compose } from 'redux';
import { browserHistory } from 'react-router';
import { routerMiddleware } from 'react-router-redux';
import multi from 'redux-multi';
import effects from 'redux-effects';
import fetch from 'redux-effects-fetch';
import createLogger from 'redux-logger';

import reducers from './reducers';


const noop = (f) => f;

const logger = createLogger({
  stateTransformer:  (state) => state.toJS()
});


export default function configureStore(initialState = Immutable.Map()) {
  let router = routerMiddleware(browserHistory);

  var store = createStore(reducers, initialState, compose(
    applyMiddleware(effects, fetch, multi, router, logger),
    window.devToolsExtension ? window.devToolsExtension() : noop
  ));

  return store;
}
