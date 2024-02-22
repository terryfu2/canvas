import React, { useRef, useEffect, useState } from 'react';
import { MapInteractionCSS } from 'react-map-interaction';

// Define the Pixel class
class Pixel {
  constructor(x, y, color) {
    this.x = x;
    this.y = y;
    this.color = color;
  }

  setColor(color) {
    this.color = color;
  }
}

// Define the PixelGrid component
const PixelGrid = ({ width, height, pixels }) => {
  const canvasRef = useRef(null);
  const [dialogCoordinates, setDialogCoordinates] = useState(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');

    // Clear the canvas
    ctx.clearRect(0, 0, width, height);

    // Draw pixels
    pixels.forEach(({ x, y, color }) => {
      ctx.fillStyle = color;
      ctx.fillRect(x, y, 10, 10); // Each pixel is 10x10
    });
  }, [width, height, pixels]);

  const handleClick = (event) => {
    const canvas = canvasRef.current;
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Find the clicked pixel
    const clickedPixel = pixels.find(pixel => x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10);

    if (clickedPixel) {
      // Open dialog with pixel coordinates
      setDialogCoordinates({ x: clickedPixel.x, y: clickedPixel.y });
      // Change color of clicked pixel
      clickedPixel.setColor('yellow'); // Change color to yellow (you can modify this)
      // Redraw canvas
      drawPixels();
    }
  };

  const drawPixels = () => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');

    // Clear the canvas
    ctx.clearRect(0, 0, width, height);

    // Draw pixels
    pixels.forEach(({ x, y, color }) => {
      ctx.fillStyle = color;
      ctx.fillRect(x, y, 10, 10); // Each pixel is 10x10
    });
  };

  return (
    <div>
      <canvas ref={canvasRef} width={width} height={height} style={{ width: '100%', height: '100%', cursor: 'pointer' }} onClick={handleClick} />
      {dialogCoordinates && (
        <div style={{ position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', backgroundColor: 'white', padding: '10px', border: '1px solid black' }}>
          <p>Clicked pixel coordinates:</p>
          <p>X: {dialogCoordinates.x/10}</p>
          <p>Y: {dialogCoordinates.y/10}</p>
          <button onClick={() => setDialogCoordinates(null)}>Close</button>
        </div>
      )}
    </div>
  );
};

// Example usage of Pixel and PixelGrid
const ExampleApp = () => {
  // Create an array of Pixel instances
  const pixels = [
    new Pixel(0, 0, 'red'),
    new Pixel(10, 10, 'blue'),
    new Pixel(20, 20, 'green')
  ];

  for(var i = 0;i<1000;i++){
    for(var j = 0;j<1000;j++){
        pixels.push(new Pixel(i*10,j*10,'white'));
    }
  }
  return (
    <div style={{ width: '100%', height: '100vh' }}>
     
      
      
        <PixelGrid width={window.innerWidth} height={window.innerHeight} pixels={pixels} />
      
    </div>
  );
};

export default ExampleApp;