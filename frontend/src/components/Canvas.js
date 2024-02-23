import React, { useRef, useEffect, useState } from 'react';
import PixelPopUp from './common/PixelPopUp';

const Canvas = ({ width, height, pixels }) => {
    const canvasRef = useRef(null);
    const [dialogCoordinates, setDialogCoordinates] = useState(null);
    
    useEffect(()=>{
        drawPixels();
    });

    const handleClick = (event) => {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
    
        const clickedPixel = pixels.find(pixel => x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10);
    
        if (clickedPixel) {
            setDialogCoordinates({ x: clickedPixel.x, y: clickedPixel.y , color: clickedPixel.color});
            
            console.log(clickedPixel.x,clickedPixel.y);
            clickedPixel.setColor('yellow'); 
            ctx.fillStyle = 'yellow'; // Set color
            ctx.fillRect(clickedPixel.x, clickedPixel.y, 10, 10); // Fill rectangle at pixel position
        }
        //drawPixels();
    };

    
    const drawPixels = () => {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
    
        ctx.clearRect(0, 0, width, height);
    
        pixels.forEach(({ x, y, color }) => {
            ctx.fillStyle = color;
            ctx.fillRect(x, y, 10, 10);
        });
    };

    const handleCloseDialog = () => {
        setDialogCoordinates(null);
    };
    
    return (
        <div>
        <canvas ref={canvasRef} width={width} height={height} style={{ width: '100%', height: '100%', cursor: 'pointer' }} onClick={handleClick} />
        {dialogCoordinates && <PixelPopUp x={dialogCoordinates.x} y={dialogCoordinates.y} color = {dialogCoordinates.color} onClose={handleCloseDialog} />}
        </div>
    );
};

export default Canvas;