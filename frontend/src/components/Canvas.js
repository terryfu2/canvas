import React, { useRef, useEffect, useState } from 'react';

const Canvas = ({ width, height, pixels }) => {
    const canvasRef = useRef(null);
    const [dialogCoordinates, setDialogCoordinates] = useState(null);
    
    useEffect(()=>{
        drawPixels();
    });
    
    const handleClick = (event) => {
        const canvas = canvasRef.current;
        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
    
        const clickedPixel = pixels.find(pixel => x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10);
    
        if (clickedPixel) {
            setDialogCoordinates({ x: clickedPixel.x, y: clickedPixel.y });
            clickedPixel.setColor('yellow'); 
            drawPixels();
        }
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
    
    return (
        <div>
        <canvas ref={canvasRef} width={width} height={height} style={{ width: '100%', height: '100%', cursor: 'pointer' }} onClick={handleClick} />
        {dialogCoordinates && (
            <div style={{ position: 'fixed', top: `${dialogCoordinates.y+80}px`, left: `${dialogCoordinates.x}px`, transform: 'translate(-50%, -50%)', backgroundColor: 'white', padding: '10px', border: '1px solid black' }}>
                <p>Clicked pixel coordinates:</p>
                <p>X: {dialogCoordinates.x/10}</p>
                <p>Y: {dialogCoordinates.y/10}</p>
                <button onClick={() => setDialogCoordinates(null)}>Close</button>
            </div>
        )}
        </div>
    );
};

export default Canvas;