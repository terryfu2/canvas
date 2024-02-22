import React, { useRef, useState, useEffect } from 'react';
import Row from './pixels/Row';
import { CurrentPixel } from './pixels/CurrentPixel';
import { MapInteractionCSS } from 'react-map-interaction';


import '../styles/drawingPanel.scss';

function Canvas({ width, height }) {
    const componentRef = useRef();
    const [xCoord, setXCoord] = useState(null);
    const [yCoord, setYCoord] = useState(null);
    const [current_color,setColor] = useState('#fff');
    const [selectedColor, setSelectedColor] = useState('#f44336');

   

    const handleHover = (x, y,color) => {
        setXCoord(x);
        setYCoord(y);
        setColor(color);
    };

    const handleColorSelect = (color) => {
        setSelectedColor(color.hex)
    }

    let rows = [];
    for (let i = 0; i < height; i++) {
        rows.push(<Row key={i} xCoord = {i} width={width} selectedColor={selectedColor} handleHover={handleHover}/>);
    }

    return (
        <MapInteractionCSS>
            <div id="drawing-panel">
                <div id="pixels" ref={componentRef}>
                    {rows}
                </div>
                <CurrentPixel xCoord={xCoord} yCoord={yCoord} current_color={current_color} setSelectedColor={handleColorSelect}/>
            </div>
        </MapInteractionCSS>
    );
}

export default Canvas;