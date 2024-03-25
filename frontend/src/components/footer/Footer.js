import React, { useRef,useState } from 'react';
import { exportComponentAsPNG } from 'react-component-export-image';
import Switch from '@mui/material/Switch';
import { IoIosInformationCircleOutline } from "react-icons/io";

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
            <h1 style={{ marginRight: 'auto' }}>canva</h1>
            <p style={{ marginRight: 'auto'}}>
                ({x / 10 || 0}, {y / 10 || 0})
            </p>
            
            <div style={{ marginRight: '20px' }}> {/* Adjust the margin-left */}
                <Switch {...label} onChange={handleSwitchChange} />
            </div>
            <div style={{ marginRight: '50px', fontSize: '30px', color: 'black' }} onMouseOver={(e) => e.target.style.color = 'blue'} onMouseOut={(e) => e.target.style.color = 'black'}>
                <IoIosInformationCircleOutline />
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