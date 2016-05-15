import Immutable from 'immutable';
import { handleActions } from 'redux-actions';

import { FetchStatus } from 'consts';


function genFetchStart() {
  return (state) => state.mergeDeep({
    ui: {
      status: FetchStatus.LOADING,
    }
  });
}

function genFetchSuccess(extra=() => ({})) {
  return (state, action) => state
    .mergeDeep(extra(state, action))
    .mergeDeep({
      data: action.payload,
      ui: {
        status: FetchStatus.SUCCESS,
        error: null
      }
    });
}

function genFetchFailure(title, message) {
  return (state, {payload: {status, statusText}}) => state.mergeDeep({
    ui: {
      status: FetchStatus.FAILED,
      error: {
        title: title,
        message: `${message} (Error ${status || '999'}: ${statusText || 'Server is unreachable'})`
      },
    }
  });
}


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

  'OVERVIEW_FETCH_START': genFetchStart(),

  'OVERVIEW_FETCH_SUCCESS': genFetchSuccess((state, action) => ({data: action.payload})),

  'OVERVIEW_FETCH_FAILURE': genFetchFailure('Could not load websites', 'Failed to load websites'),

  'OVERVIEW_ENQUEUE_WEBSITE_START': genFetchStart(),

  'OVERVIEW_ENQUEUE_WEBSITE_SUCCESS': genFetchSuccess(),

  'OVERVIEW_ENQUEUE_WEBSITE_FAILURE': genFetchFailure('Could not enqueue website', 'Failed to enqueue website'),
}, DEFAULT_STATE);