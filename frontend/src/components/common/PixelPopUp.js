import React, { useState } from 'react';
import { CirclePicker,CompactPicker,SketchPicker } from 'react-color';

const PixelPopUp = ({ x, y, color,onClose ,onConfirm}) => {
    const [showColorPicker, setShowColorPicker] = useState(false);
    const [selectedColor, setSelectedColor] = useState(color);

    const ColorBoxStyle = (color) => ({
        backgroundColor: color, 
        width: '30px', 
        height: '30px', 
        border: '1px solid black', 
        marginLeft: '10px' 
    });

    const PopUpStyle = (topPosition,x) =>({
        position: 'fixed', 
        top: topPosition, 
        left: `${x}px`, 
        transform: 'translate(-50%, -50%)', 
        backgroundColor: 'white', 
        padding: '10px', 
        border: '1px solid black', 
        color: 'black', 
        display: 'flex', 
        flexDirection: 'column', 
        alignItems: 'center'
    });

    const handleColorBoxClick = () => {
        setShowColorPicker(!showColorPicker); 
    };
    const handleColorChange = (newColor) => {
        setSelectedColor(newColor.hex);
    };

    const handleConfirm = () => {
        onConfirm(selectedColor);
        setShowColorPicker(false); 
    };


    // Check if the dialog fits within the screen height
    const fitsInScreen = (y + 210) <= window.innerHeight;
    // Calculate the top position based on whether it fits or not
    const topPosition = fitsInScreen ? `${y + 90}px` : `${y - 80}px`;

    return (
        <div style={PopUpStyle(topPosition,x)}>
            <div style={{ display: 'flex', alignItems: 'center', marginBottom: '10px' }}>
                <div style={{ display: 'flex', flexDirection: 'column' }}>
                    <p>X   &nbsp; {x / 10}</p>
                    <p>Y   &nbsp; {y / 10}</p>
                </div>
                <div style={ColorBoxStyle(color)}  onClick={handleColorBoxClick}></div>
            </div>
            {showColorPicker && <SketchPicker color={selectedColor} onChange={handleColorChange} disableAlpha = {true}/>}
            <div>
                <button onClick={onClose}>Close</button>
                <button onClick={handleConfirm} style={{ marginLeft: '10px' }}>Remove</button>
                <button onClick={handleConfirm} style={{ marginLeft: '10px' }}>Confirm</button>

            </div>
        </div>
    );
};

export default PixelPopUp;