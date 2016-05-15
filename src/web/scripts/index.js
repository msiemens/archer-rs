import React from 'react';
import { render } from 'react-dom';

import { Provider } from 'react-redux';
import { browserHistory, Router, Route, IndexRoute } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';

import 'expose?$!expose?jQuery!jquery';
import 'jquery-serializejson';
import 'semantic-ui/dist/semantic.css';
import 'semantic-ui/dist/semantic.js';

import configureStore from './store';
import App from './app/containers/App';
import OverviewPage from 'overview/containers/OverviewPage';

export const store = configureStore();

const history = syncHistoryWithStore(browserHistory, store, {
  selectLocationState: state => state.get('route') || {}
});

render(
  <Provider store={store}>
    <Router history={history}>
      <Route component={App} path='/'>
        <IndexRoute component={OverviewPage} />
        <Route path='tags' />
        <Route path='queue' />
      </Route>
    </Router>
  </Provider>, document.getElementById('app')
);
