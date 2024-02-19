import React, { useState } from 'react';
import '../styles/pixel.scss';

function Pixel({ xCoord, yCoord, selectedColor, handleHover }) {
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

  const handleMouseEnter = () => {
    // Get the coordinates of the pixel
    // For example, assuming x and y are obtained somehow
    const x = xCoord; // Get the x coordinate
    const y = yCoord; // Get the y coordinate
    const color = pixelColor;
    handleHover(x, y,color);
  };
  const handleMouseEnterWrapper = () => {
    //changeColorOnHover();
    handleMouseEnter();
  };

  //onMouseLeave={resetColor}
  return (
    <div 
      className='pixel' 
      style={{backgroundColor: pixelColor}}
      onClick={applyColor} 
      onMouseEnter={handleMouseEnterWrapper} 
      
      onDoubleClick={removeColor}
    >

    </div>
  )
}

export default Pixel;