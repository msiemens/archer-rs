import { bind } from 'redux-effects';
import { fetch } from 'redux-effects-fetch';
import { createAction } from 'redux-actions';

import { API_ROOT } from 'consts';


export function fetchOverview() {
  let url = `${API_ROOT}/overview`;

  return [
    overviewFetchStart(),
    bind(
      fetch(url, {
        method: 'GET'
      }),
      ({value}) => overviewFetchSuccess(value),
      ({status, statusText}) => overviewFetchFailure({status, statusText})
    )
  ];
}

const overviewFetchStart = createAction('OVERVIEW_FETCH_START');
const overviewFetchSuccess = createAction('OVERVIEW_FETCH_SUCCESS');
const overviewFetchFailure = createAction('OVERVIEW_FETCH_FAILURE');

export const setFilter = createAction('OVERVIEW_SET_FILTER');


export function enqueueWebsite(data) {
  return [
    overviewEnqueueWebsiteStart(),
    bind(
      fetch(`${API_ROOT}/overview`, {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: data
      }),
      () => overviewEnqueueWebsiteSuccess(),
      ({status, statusText}) => overviewEnqueueWebsiteFailure({status, statusText})
    )
  ];
}

const overviewEnqueueWebsiteStart = createAction('OVERVIEW_ENQUEUE_WEBSITE_START');
const overviewEnqueueWebsiteSuccess = createAction('OVERVIEW_ENQUEUE_WEBSITE_SUCCESS');
const overviewEnqueueWebsiteFailure = createAction('OVERVIEW_ENQUEUE_WEBSITE_FAILURE');