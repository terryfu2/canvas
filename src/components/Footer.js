import React from "react";

const TextStyle = {
    position: 'fixed',
    bottom: '3em',
    right: '5em',
    //opacity: '1',
    fontsize: '20px',
    zIndex: '9999',
    color: 'black',
    fontWeight: 'bold'
    
};

const ColorBoxStyle = (currentColor) => ({
    width: '40px',
    height: '40px',
    backgroundColor: currentColor,
    display: 'inline-block',
    marginLeft: '5px',
    border: '1px solid black'
});

export const Footer = ({ xCoord, yCoord,current_color }) => (
    <div style={TextStyle}>
        <div style={ColorBoxStyle(current_color)}></div>
        <br></br>
        {xCoord},{yCoord}
        
        
       
    </div>
    
);