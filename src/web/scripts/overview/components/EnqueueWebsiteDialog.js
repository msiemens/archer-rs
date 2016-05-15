import React from 'react';

export default class EnqueueWebsiteDialog extends React.Component {
  static propTypes = {
    onEnqueue: React.PropTypes.func.isRequired
  };

  componentDidMount() {
    jQuery(this.modal)
      .modal({
        detachable: false,
        onApprove: () => {
          const $form = $(this.form);
          if ($form.form('is valid')) {
            this.props.onEnqueue($(this.form).serializeJSON());
          } else {
            // Re-center modal
            jQuery(this.modal).modal('refresh');
            return false;
          }
        },
        onVisible: () => {
          jQuery(this.dropdown).dropdown();
        }
      })
      .modal('show');

    jQuery(this.form).form({
      inline: true,
      fields: {
        title: 'empty',
        url: ['empty', 'url'],
        tags: 'regExp[/^(|\\d(,\\d)*)$/]'
      },
    });

    jQuery(this.form).on('submit', () => false);
  }

  componentWillUnmount() {
    jQuery(this.modal)
      .modal('hide');
  }

  close(e) {
    e.preventDefault();
    // this.props.onEnqueue();
  }

  render() {
    const { tags } = this.props;

    let tagList = Object.keys(tags).map((i) => {
      const tag = tags[i];

      return (<div className='item' key={i} data-value={i}>
        <div className={'ui empty circular label ' + (tag.color || '')}></div> {' '}
        {tag.name}
      </div>);
    });

    return (
      <div className='ui modal small' ref={(node) => this.modal = node}>
        <div className='header'>Add Website</div>
        <div className='content'>
          <form className='ui form' ref={(node) => this.form = node}>
            <div className='required field'>
              <label>Title</label>
              <input name='title' placeholder='' type='text' />
            </div>
            <div className='required field'>
              <label>URL</label>
              <input name='url' placeholder='' type='text' />
            </div>
            <div className='field'>
              <label>Tags</label>
              <div className='ui dropdown fluid search multiple selection' ref={(node) => this.dropdown = node}>
                <input name='tags' type='hidden' />
                <i className='dropdown icon' />
                <div className='default text'>None</div>
                <div className='menu'>
                  {tagList}
                </div>
              </div>
            </div>
          </form>
        </div>
        <div className='actions'>
          <div className='ui black deny button'>Cancel</div>
          <div className='ui positive right labeled icon button'>
            Add Website
            <i className='checkmark icon' />
          </div>
        </div>
      </div>
    );
  }
}
