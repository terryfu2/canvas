import React, { useState } from 'react';
import '../styles/pixel.scss';

function Pixel({ xCoord, yCoord, selectedColor }) {
  const [pixelColor, setPixelColor] = useState('#fff');
  const [oldColor, setOldColor] = useState(pixelColor);
  const [canChangeColor, setCanChangeColor] = useState(true);

  const applyColor = () => {
    setPixelColor(selectedColor);
    setCanChangeColor(false);
    console.log(xCoord + ' ' + yCoord + ' ' + selectedColor);
  };

  const changeColorOnHover = () => {
    setOldColor(pixelColor);  
    setPixelColor(selectedColor);
  };

  const resetColor = () => {
    if (canChangeColor) {
      setPixelColor(oldColor);
    }

    setCanChangeColor(true);
  };


  const removeColor = () => {
    setPixelColor('#fff');
  };

  return (
    <div 
      className='pixel' 
      style={{backgroundColor: pixelColor}}
      onClick={applyColor} 
      onMouseEnter={changeColorOnHover} 
      onMouseLeave={resetColor}
      onDoubleClick={removeColor}
    >

    </div>
  )
}

export default Pixel;