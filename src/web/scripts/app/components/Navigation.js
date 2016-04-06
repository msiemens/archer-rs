import React from 'react';
import { Link } from 'react-router';


const NavItem = ({url, children}) => (
  <Link to={url} className='item' activeClassName='active'>{children}</Link>
);


export default () => (
  <div className='ui menu large stackable'>
    <div className='ui container'>
      <div className='header item'>
        Archer
      </div>
      <NavItem url='/'>Websites</NavItem>
      <NavItem url='tags'>Tags</NavItem>
      <NavItem url='queue'>Queue</NavItem>
    </div>
  </div>
);
