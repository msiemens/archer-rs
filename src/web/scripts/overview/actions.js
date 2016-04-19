import { bind } from 'redux-effects';
import { fetch } from 'redux-effects-fetch';
import { createAction } from 'redux-actions';

import { API_ROOT } from 'consts';


export function fetchOverview(filter=null) {
  let url = `${API_ROOT}/overview`;

  if (filter !== null) {
    url += `?tag=${filter}`;
  }

  return [
    overviewFetchStart(filter),
    bind(
      fetch(url, {
        method: 'GET'
      }),
      ({value}) => overviewFetchSuccess(value),
      ({status, statusText}) => overviewFetchFailure({status, statusText})
    )
  ];
}

export const overviewFetchStart = createAction('OVERVIEW_FETCH_START');
export const overviewFetchSuccess = createAction('OVERVIEW_FETCH_SUCCESS');
export const overviewFetchFailure = createAction('OVERVIEW_FETCH_FAILURE');