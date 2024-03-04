import React, { useState } from 'react';
import {SketchPicker } from 'react-color';

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

    const PopUpStyle = () =>({
        
        position: 'absolute',
        top: '25%', 
        left: '50%',
        transform: 'translate(-25%, -50%)', 
        backgroundColor: 'white', 
        padding: '10px', 
        border: '1px solid black', 
        color: 'black', 
        zIndex: 9999,
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        flexDirection: 'column'
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


    return (
        <div style={PopUpStyle()}>
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
                <button onClick={handleConfirm} style={{ marginLeft: '10px' }}>Confirm</button>

            </div>
        </div>
    );
};

export default PixelPopUp;