import React, { useRef, useState, useEffect } from 'react';
import Row from './Row';

import '../styles/drawingPanel.scss';

function Canvas({ width, height }) {
    const componentRef = useRef();
    let rows = [];



    for (let i = 0; i < height; i++) {
        rows.push(<Row key={i} xCoord = {i} width={width} selectedColor={'#f44336'} />);
    }

    return (
        <div id="drawing-panel">
            <div id="pixels" ref={componentRef}>
                {rows}
            </div>
        </div>
    );
}

export default Canvas;