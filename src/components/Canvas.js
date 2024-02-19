import React, { useRef, useState, useEffect } from 'react';
import Row from './Row';
import { Footer } from './Footer';

import '../styles/drawingPanel.scss';

function Canvas({ width, height }) {
    const componentRef = useRef();
    const [xCoord, setXCoord] = useState(null);
    const [yCoord, setYCoord] = useState(null);
    const [current_color,setColor] = useState(null);
    let rows = [];

    const handleHover = (x, y,color) => {
        setXCoord(x);
        setYCoord(y);
        setColor(color);
    };

    for (let i = 0; i < height; i++) {
        rows.push(<Row key={i} xCoord = {i} width={width} selectedColor={'#f44336'} handleHover={handleHover}/>);
    }

    return (
        <div id="drawing-panel">
            <div id="pixels" ref={componentRef}>
                {rows}
            </div>
            <Footer xCoord={xCoord} yCoord={yCoord} current_color={current_color}/>
        </div>
    );
}

export default Canvas;