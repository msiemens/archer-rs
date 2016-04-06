// See https://github.com/gajus/redux-immutable#using-with-react-router-redux
import Immutable from 'immutable';
import { routerReducer, LOCATION_CHANGE } from 'react-router-redux';

const initialState = Immutable.fromJS({});

export const route = (state = initialState, action = {}) => (
  (action.type == LOCATION_CHANGE)
    ? state.set('route', routerReducer(state.get('route'), action))
    : state
);
