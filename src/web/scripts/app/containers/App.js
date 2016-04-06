import React from 'react';
import { connect } from 'react-redux';

import Navigation from 'app/components/Navigation';


export class App extends React.Component {
  static propTypes = {
    children: React.PropTypes.element
  };

  render() {
    return (
      <div>
        <Navigation />
        <div className='ui container'>
          {this.props.children}
        </div>
      </div>
    );
  }
}


export default connect()(App);
