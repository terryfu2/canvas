import React, { useRef, useEffect, useState } from 'react';
import PixelPopUp from './common/PixelPopUp';
import Footer from './footer/Footer';
import { MapInteractionCSS } from 'react-map-interaction';


const Canvas = ({ width, height, pixels }) => {
    const canvasRef = useRef(null);
    const [dialogCoordinates, setDialogCoordinates] = useState(null);
    const [hoveredPixel, setHoveredPixel] = useState({ x: 0, y: 0 });    
    const [clickedPixel, setClickedPixel] = useState(null);

    //only update canvas is there is a changes to array
    useEffect(() => {
        drawPixels();
    }, [pixels]);

    const drawPixels = () => {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
    
        ctx.clearRect(0, 0, width, height);
    
        pixels.forEach(({ x, y, color }) => {
            ctx.fillStyle = color;
            ctx.fillRect(x, y, 10, 10);
        });
    };

    const handleClickPixel = (event) => {
        const canvas = canvasRef.current;
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const x = (event.clientX - rect.left) * scaleX;
        const y = (event.clientY - rect.top) * scaleY;
    
        const clickedPixel = pixels.find(pixel => x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10);
        if (clickedPixel) {
            setDialogCoordinates({ x: clickedPixel.x, y: clickedPixel.y, color: clickedPixel.color });
        }
        setClickedPixel(clickedPixel);
    };

    const handleMouseMove = (event) => {
        const canvas = canvasRef.current;
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const x = (event.clientX - rect.left) * scaleX;
        const y = (event.clientY - rect.top) * scaleY;
    

        const pixel = pixels.find(pixel => x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10);
        setHoveredPixel(pixel);
    };

    const handleCloseDialog = () => {
        setDialogCoordinates(null);
        setClickedPixel(null);
    };

    const handleConfirm = (color) => {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');

        clickedPixel.setColor(color); 
        ctx.fillStyle = color; // Set color
        ctx.fillRect(clickedPixel.x, clickedPixel.y, 10, 10); 

        setDialogCoordinates(null);
        setClickedPixel(null);
    }

    
    return (
        <div>
            <MapInteractionCSS>

            <canvas ref={canvasRef} width={width} height={height} style={{ width: '100%', height: '100%', cursor: 'pointer' }} onClick={handleClickPixel} onMouseMove={handleMouseMove}/>
            {dialogCoordinates && <PixelPopUp x={dialogCoordinates.x} y={dialogCoordinates.y} color = {dialogCoordinates.color} onClose={handleCloseDialog} onConfirm={handleConfirm}/>}
            
            </MapInteractionCSS>
        
            <Footer x={hoveredPixel.x} y={hoveredPixel.y}></Footer>
        </div>
    );
};

export default Canvas;