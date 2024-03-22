import React, { useRef,useState } from 'react';
import { exportComponentAsPNG } from 'react-component-export-image';
import Switch from '@mui/material/Switch';

const label = { inputProps: { 'aria-label': 'Switch demo' } };

function Footer({ x,y, sendTimeout }) {
    const componentRef = useRef();
    const [timeoutEnabled, setTimeoutEnabled] = useState(false);

    const handleSwitchChange = () => {
        setTimeoutEnabled(!timeoutEnabled);
        sendTimeout(timeoutEnabled);
    };

    return (
        <div style={footerStyle}>
            <h1 style={{ marginRight: 'auto' }}>canvas</h1>
            <p style={{ marginRight: 'auto'}}>
                ({x / 10 || 0}, {y / 10 || 0})
            </p>
            <div style={{ marginRight: '100px' }}> {/* Adjust the margin-left */}
                <Switch {...label} onChange={handleSwitchChange} />
            </div>

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