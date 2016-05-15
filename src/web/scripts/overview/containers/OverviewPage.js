import React from 'react';
import Helmet from 'react-helmet';
import { connect } from 'react-redux';

import { FetchStatus, TITLE_TEMPLATE } from 'consts';
import { fetchOverview, setFilter, enqueueWebsite } from '../actions';
import EnqueueWebsiteDialog from '../components/EnqueueWebsiteDialog';


export class OverviewPage extends React.Component {
  static contextTypes = {
    router: React.PropTypes.object.isRequired
  };

  componentWillMount() {
    this.props.dispatch(fetchOverview());
  }

  componentDidMount() {
    jQuery(this.filter).dropdown({
      onChange: (value) => {
        this.props.dispatch(setFilter(value));
      }
    });
  }

  componentDidUpdate() {
    jQuery(this.filter).dropdown('refresh');
  }

  renderOverview(websites) {
    return (
      <table className='ui table fixed single line'>
        <thead className='full-width'>
          <tr>
            <th>Title</th>
            <th>URL</th>
            <th>Tags</th>
          </tr>
        </thead>
        <tbody>
        {websites.map((website) => (
          <tr key={website.title}>
            <td>{website.title}</td>
            <td>{website.url}</td>
            <td>
              {website.tags.map((tag) => (
                <a key={tag.name} className={'ui horizontal label ' + (tag.color || '')}>{tag.name}</a>
              ))}
            </td>
          </tr>
        ))}
        </tbody>
      </table>
    );
  }

  render() {
    const { data, status, error } = this.props;
    let contents, tags, enqueueWebsiteDialog;

    switch (status) {
      case FetchStatus.LOADING:
        contents = <div className='ui active text loader'>Loading</div>;
        break;
      case FetchStatus.FAILED:
        contents = (<div className='ui negative icon message'>
          <i className='lightning icon'></i>
          <div className='content'>
            <div className='header'>
              {error.title}
            </div>
            {error.message}
          </div>
        </div>);
        break;
      case FetchStatus.SUCCESS:
        enqueueWebsiteDialog = (<EnqueueWebsiteDialog tags={data.tags}
                                                      onEnqueue={(data) => this.props.dispatch(enqueueWebsite(data))} />);

        if (data.websites.length > 0) {
          contents = this.renderOverview(data.websites);
        } else {
          contents = (<div className='ui icon message'>
            <i className='circle notched icon'></i>
            <div className='content'>
              <div className='header'>
                No websites
              </div>
              No websites have been downloaded/queued yet.
            </div>
          </div>);
        }

        tags = Object.keys(data.tags).map((i) => {
          const tag = data.tags[i];

          return (<div key={tag.name} className='item' data-value={i}>
            <div className={'ui empty circular label ' + (tag.color || '')}></div> {' '}
            {tag.name}
          </div>);
        });
        break;
    }

    return (
      <div>
        <Helmet title='Overview' titleTemplate={TITLE_TEMPLATE}/>

        {enqueueWebsiteDialog}

        <div className='ui floating dropdown button labeled icon' ref={(node) => this.filter = node}>
          <i className='filter icon'></i>
          <span className='text'>Filter by Tag</span>
          <div className='menu'>
            <div className='item' data-value=''>
              Show All
            </div>
            {tags}
          </div>
        </div>

        <div className='ui right floated labeled icon button'>
          <i className='plus icon'></i> Add Website
        </div>

        {contents}
      </div>);
  }
}

function mapStateToProps(state) {
  state = state.toJS()['overview'];

  if (state.ui.status == FetchStatus.SUCCESS) {
    state.data.websites = state.data.websites.filter((website) => {
      if (state.ui.filter != null && state.ui.filter != '') {
        return website.tags.indexOf(parseInt(state.ui.filter, 10)) != -1;
      } else {
        return true;
      }
    }).map((website) => ({
      title: website.title,
      url: website.url,
      tags: website.tags.map((tagId) => state.data.tags[tagId])
    }));

    return {
      data: state.data,
      status: state.ui.status,
      error: state.ui.error
    };
  } else {
    return {
      status: state.ui.status,
      error: state.ui.error
    };
  }
}


export default connect(mapStateToProps)(OverviewPage);

/*
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
*/