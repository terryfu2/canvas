import React, { useState } from 'react';
import {SketchPicker } from 'react-color';
import "../../styles/button.scss"

const PixelPopUp = ({ x, y, color,onClose ,onConfirm,disabledConfirm}) => {
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
                    <p>X  &nbsp; <strong>{x / 10}</strong></p>
                    <p>Y  &nbsp; <strong>{y / 10}</strong></p>
                    <p>Hex   &nbsp; <strong>{color}</strong></p>
                </div>
                <div style={ColorBoxStyle(color)}  onClick={handleColorBoxClick}></div>
            </div>
            {showColorPicker && <SketchPicker color={selectedColor} onChange={handleColorChange} disableAlpha = {true}/>}
            <div>
                <button onClick={onClose} className = 'button-89'>Close</button>
                <button className = 'button-89' onClick={handleConfirm} disabled = {disabledConfirm} style={{ marginLeft: '10px' } }>Confirm</button>

            </div>
        </div>
    );
};

export default PixelPopUp;