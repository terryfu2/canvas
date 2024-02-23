import React, { useRef } from 'react';
import { exportComponentAsPNG } from 'react-component-export-image';

function Footer(hoveredPixel) {
    const componentRef = useRef();

    return (
        <div style={footerStyle}>
            <h1 style={{ marginRight: 'auto' }}>canvas</h1>
            <p style={{ marginRight: 'auto'}}>({hoveredPixel.x/10}, {hoveredPixel.y/10}) </p>

            {/*<button
                className='button-outline'
                onClick={() => exportComponentAsPNG(componentRef)}
            >
                Export as PNG
            </button>*/}

        </div>
    );
}

const footerStyle = {
    display: 'flex', 
    justifyContent: 'space-between', 
    alignItems: 'center', 
    position: 'fixed',
    bottom: 0,
    left: 0,
    width: '100%',
    height: '5%', 
    background: '#F4EBD0',
    color: '#886F68',
    textAlign: 'center',
    padding: '10px'
};

export default Footer;