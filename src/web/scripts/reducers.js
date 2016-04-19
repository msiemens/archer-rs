// import Immutable from 'immutable';
// import { reducer as form } from 'redux-form';
import { combineReducers } from 'redux-immutable';

// import helpers from './helpers/reducer';
import overview from './overview/reducer';
import { route } from './app/reducer';


// function immutableize(reducer) {
//     return (state, action) => Immutable.fromJS(reducer(state && state.toJS(), action));
// }


export default combineReducers({
  overview,
  route,
  // form: immutableize(form)
});
