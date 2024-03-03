
import React, { useRef, useEffect, useState } from 'react';
import Color from "colorjs.io";
import PixelPopUp from './common/PixelPopUp';
import Footer from './footer/Footer';
import { MapInteractionCSS } from 'react-map-interaction';

const Canvas = ({onPixelChange, width, height, pixels }) => {
    const canvasRef = useRef(null);

    const [dialogCoordinates, setDialogCoordinates] = useState(null);
    const [hoveredPixel, setHoveredPixel] = useState({ x: 0, y: 0 });    
    const [clickedPixel, setClickedPixel] = useState(null);

    

    //only update canvas is there is a changes to array
    useEffect(() => {
        fetch("/api/canvas")
        .then((res) => res.json())
            .then((res) => {
                for (let pixel of res) {
                    //console.log(pixel)
                    pixels.map(el => el.x == pixel.x*10 && el.y == pixel.y*10 ? el.color = `#${pixel.colour.toString(16)}` : el);
                    //console.log(pixels.find(el => el.x == pixel.x*10 && el.y == pixel.y*10))
                }
                drawPixels();
            })
            .catch(console.error);
        
        drawPixels();

        
    }, [pixels]);

    const drawPixels = () => {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
    
        ctx.clearRect(0, 0, width, height);
        
        pixels.forEach(({ x, y, color }) => {
            ctx.fillStyle = color;
            

            if (color == '#0') {
                //console.log(color);
                ctx.fillStyle = 'black'; // Set color
            }

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
        let colorNum = Number(`0x${(new Color(color).toString({format: "hex"})).substring(1)}`); // there must be a better way
        //console.log(colorNum);
        ctx.fillRect(clickedPixel.x, clickedPixel.y, 10, 10);
        onPixelChange(clickedPixel.x, clickedPixel.y, colorNum);

        setDialogCoordinates(null);
        setClickedPixel(null);
    }

    
    return (
        <div>
            <MapInteractionCSS
                
            >

                <canvas ref={canvasRef} width={width} height={height} style={{ width: '100%', height: '100%', cursor: 'pointer' }} onClick={handleClickPixel} onMouseMove={handleMouseMove}/>
            
            </MapInteractionCSS>

            {dialogCoordinates && <PixelPopUp x={dialogCoordinates.x} y={dialogCoordinates.y} color = {dialogCoordinates.color} onClose={handleCloseDialog} onConfirm={handleConfirm}/>}

        
            <Footer x={hoveredPixel ? hoveredPixel.x : 0} y={hoveredPixel ? hoveredPixel.y : 0}></Footer>
        </div>
    );
};

export default Canvas;
