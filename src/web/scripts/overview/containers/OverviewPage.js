import React from 'react';
import { connect } from 'react-redux';

import 'semantic-ui/dist/semantic.js';

console.log(jQuery);
console.log(jQuery.fn.dropdown);

export class OverviewPage extends React.Component {
  componentDidMount() {
    console.log(jQuery.fn.dropdown);
    jQuery('.ui.dropdown').dropdown({
      onChange: (value) => {}
    });
  }

  componentDidUpdate() {
    jQuery('.ui.dropdown').dropdown('refresh');
  }

  render() {
    return (
      <div>
        <div className='ui floating dropdown button labeled icon'>
          <i className='filter icon'></i>
          <span className='text'>Filter by Tag</span>
          <div className='menu'>
            <div className='item'>
              <div className='ui empty circular label'></div>
              Interessant
            </div>
            <div className='item'>
              <div className='ui empty blue circular label'></div>
              Glaube
            </div>
          </div>
        </div>

        <div className='ui right floated labeled icon button'>
          <i className='plus icon'></i> Add Website
        </div>

        <table className='ui table fixed single line'>
          <thead className='full-width'>
            <tr>
              <th>Title</th>
              <th>URL</th>
              <th>Tags</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>Doomsday planning for less crazy folk</td>
              <td>http://lcamtuf.coredump.cx/prep/</td>
              <td><a className='ui horizontal label'>Interessant</a></td>
            </tr>
            <tr>
              <td>The Long Silence</td>
              <td>http://www.ldolphin.org/silence.html</td>
              <td><a className='ui horizontal label'>Interessant</a><a className='ui horizontal label blue'>Glaube</a></td>
            </tr>
            <tr>
              <td>Understanding Depression</td>
              <td>http://health.howstuffworks.com/mental-health/depression/facts/understanding-depression-ga.htm</td>
              <td><a className='ui horizontal label'>Interessant</a></td>
            </tr>
          </tbody>
        </table>
      </div>);
  }
}


export default connect()(OverviewPage);
