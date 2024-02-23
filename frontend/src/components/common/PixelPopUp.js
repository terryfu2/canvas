import React from 'react';

const PixelPopUp = ({ x, y, color,onClose }) => {

    const ColorBoxStyle = (currentColor) => ({
        width: '40px',
        height: '40px',
        backgroundColor: currentColor,
        display: 'inline-block',
        marginLeft: '5px',
        border: '1px solid black'
    });


    // Check if the dialog fits within the screen height
    const fitsInScreen = (y + 130) <= window.innerHeight;
    // Calculate the top position based on whether it fits or not
    const topPosition = fitsInScreen ? `${y + 90}px` : `${y - 80}px`;

    return (
        <div style={{ position: 'fixed', top: topPosition, left: `${x}px`, transform: 'translate(-50%, -50%)', backgroundColor: 'white', padding: '10px', border: '1px solid black' }}>
            <p>Clicked pixel coordinates:</p>
            <p>X: {x / 10}</p>
            <p>Y: {y / 10}</p>
            <div style={ColorBoxStyle(color)}></div>
            <button onClick={onClose}>Close</button>
            <button onClick={onClose}>Confirm</button>
        </div>
    );
};

export default PixelPopUp;