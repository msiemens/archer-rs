import Immutable from 'immutable';
import { handleActions } from 'redux-actions';

import { FetchStatus } from 'consts';


export const DEFAULT_STATE = Immutable.fromJS({
  data: {},
  ui: {
    status: null,
    error: null,
    filter: null
  }
});


export default handleActions({
  'OVERVIEW_SET_FILTER': (state, {payload: filter}) => state.mergeDeep({
    ui: { filter }
  }),
  'OVERVIEW_FETCH_START': (state) => state.mergeDeep({
    ui: {
      status: FetchStatus.LOADING,
    }
  }),
  'OVERVIEW_FETCH_SUCCESS': (state, action) => state.mergeDeep({
    data: action.payload,
    ui: {
      status: FetchStatus.SUCCESS,
      error: null
    }
  }),
  'OVERVIEW_FETCH_FAILURE': (state, {payload: {status, statusText}}) => state.mergeDeep({
    ui: {
      status: FetchStatus.FAILED,
      error: {
        title: 'Could not load websites',
        message: `Failed to load websites (Error ${status || '999'}: ${statusText || 'Server is unreachable'})`
      },
    }
  }),
}, DEFAULT_STATE);