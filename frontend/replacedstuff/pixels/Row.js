import React from 'react';
import Pixel from './Pixel';
import '../../styles/row.scss'; 

function Row({ xCoord, width, selectedColor ,handleHover }) {
  let pixels = [];

  for (let i = 0; i < width; i++) {
    pixels.push(<Pixel key={i} xCoord = {xCoord} yCoord = {i} selectedColor={selectedColor} handleHover={handleHover}/>);
  }

  return (
    <div className='row'>{pixels}</div>
  )
}

export default Row;