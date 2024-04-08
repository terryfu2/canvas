import Color from "colorjs.io";
import React, { useEffect, useRef, useState } from "react";
import { MapInteractionCSS } from "react-map-interaction";
import PixelPopUp from "./common/PixelPopUp";
import Footer from "./footer/Footer";

//canvas component, contains actual canvas, footer and dialog
const Canvas = ({ setPixel,isError, width, height, pixels,primary }) => {
  const canvasRef = useRef(null); 

  const [dialogCoordinates, setDialogCoordinates] = useState(null);
  const [hoveredPixel, setHoveredPixel] = useState({ x: 0, y: 0 });
  const [clickedPixel, setClickedPixel] = useState(null);
  const [confirmClicked, setConfirmClicked] = useState(false);
  const [timeoutEnabled, setTimeoutEnabled] = useState(false);

  const [canClickPixel,setCanClickPixel] = useState(true);
  const [isMouseMoved,setIsMouseMoved] = useState(false);

  const [primaryId,setPrimaryId] = useState(null);

  const [mapState, setMapState] = useState({
    scale: 0.8,
    translation: { x: 25, y: 25 },
  });

  useEffect(() => {
    setPrimaryId(prevPrimaryId => primary);
}, [primary]);

  // Redraw the canvas when the pixel data changes
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    
    ctx.clearRect(0, 0, width, height);

    pixels.forEach(({ x, y, color }) => {
      ctx.fillStyle = color;

      if (color === "#0") {
        ctx.fillStyle = "black";
      }

      ctx.fillRect(x, y, 10, 10);
    });
  }, [pixels, height, width]);

  function rgbToHex(r, g, b) {
    if (r > 255 || g > 255 || b > 255) throw "Invalid color component";
    return ((r << 16) | (g << 8) | b).toString(16);
  }

  //pixel is clicked, get information
  const handleClickPixel = (event) => {

    if(!canClickPixel){
        return;
    }
    const rect = event.target.getBoundingClientRect();
    const clickedX = (event.clientX - rect.left) / mapState.scale;
    const clickedY = (event.clientY - rect.top) / mapState.scale;

    const newScale = 13; // Zoom scale
    const newTranslationX = -(clickedX * newScale - event.clientX);
    const newTranslationY = -(clickedY * newScale - event.clientY);

    setMapState({
      scale: newScale,
      translation: { x: newTranslationX, y: newTranslationY },
    });

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    //const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const x = (event.clientX - rect.left) * scaleX;
    const y = (event.clientY - rect.top) * scaleY;

    const clickedPixel = pixels.find(
      (pixel) =>
        x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10
    );

    //get color hex from clicked pixel based on canvas
    var p = ctx.getImageData(x, y, 1, 1).data;
    var hex = "#" + ("000000" + rgbToHex(p[0], p[1], p[2])).slice(-6);

    if (clickedPixel) {
      setDialogCoordinates({
        x: clickedPixel.x,
        y: clickedPixel.y,
        color: hex,
      });
    }
    setClickedPixel(clickedPixel);
  };
  //track mouse position to update hover in footer
  const handleMouseMove = (event) => {
    setIsMouseMoved(true);

    const canvas = canvasRef.current;
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const x = (event.clientX - rect.left) * scaleX;
    const y = (event.clientY - rect.top) * scaleY;

    const pixel = pixels.find(
      (pixel) =>
        x >= pixel.x && x < pixel.x + 10 && y >= pixel.y && y < pixel.y + 10
    );
    setHoveredPixel(pixel);
  };

  const handleCloseDialog = () => {
    setDialogCoordinates(null);
    setClickedPixel(null);
  };
  //send pixel updated info to proxy
  const handleConfirm = (color) => {
    /*
    if(clickedPixel.color == 'white' && color == '#ffffff' || clickedPixel.color == color){
        setDialogCoordinates(null);
        setClickedPixel(null);
        return;
    }*/
    setConfirmClicked(true);
    console.log(`Confirm color ${color}`);
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    clickedPixel.setColor(color);
    ctx.fillStyle = color; // Set color
    let colorNum = 0;
    if (color.startsWith('#')) {
      colorNum = Number(
        `0x${color.substring(1)}`
      );
    } else {
      colorNum = Number(
        `0x${new Color(color).toString({ format: "hex" }).substring(1)}`
      ); // there must be a better way
    }

    ctx.fillRect(clickedPixel.x, clickedPixel.y, 10, 10);
    setPixel(clickedPixel.x, clickedPixel.y, colorNum);

    setDialogCoordinates(null);
    setClickedPixel(null);
    isError();
    if(timeoutEnabled){
        setTimeout(() => {
            setConfirmClicked(false);
        }, 9000);     
    }
    if(!timeoutEnabled){
        setConfirmClicked(false);
    }
  };

  const handleSwitchChange = (data) => {
    setTimeoutEnabled(!timeoutEnabled);
    console.log(timeoutEnabled);

  };

  //bad attempt at trying to fix error in mouse click being registered when dragging
  const handleDragStart = (event) =>{

    setIsMouseMoved(false);
    setCanClickPixel(false);
  }
  const handleDragEnd = (event) =>{

    if(isMouseMoved){
        setTimeout(() => {
            setCanClickPixel(true);
            setIsMouseMoved(false)
        }, 1000); 
    }
    else{
        setCanClickPixel(true);

    }
  }

  return (
    <div>
      <MapInteractionCSS
        showControls
        value={mapState} 
        onChange={(value) => setMapState(value)} 
        minScale={0.6}
        maxScale={20}
      >
        <canvas
          ref={canvasRef}
          width={width}
          height={height}
          draggable = {true}
          style={{
            width: "100%",
            height: "100%",
            cursor: "pointer",
            imageRendering: "pixelated",
          }}
          onClick={handleClickPixel}
          onMouseMove={handleMouseMove}
          onMouseDown = {handleDragStart}
          onMouseUp = {handleDragEnd}
        />
      </MapInteractionCSS>

      {dialogCoordinates && (
        <PixelPopUp
          x={dialogCoordinates.x}
          y={dialogCoordinates.y}
          color={dialogCoordinates.color}
          onClose={handleCloseDialog}
          onConfirm={handleConfirm}
          disabledConfirm = {confirmClicked}
        />
      )}

      <Footer
        x={hoveredPixel ? hoveredPixel.x : 0}
        y={hoveredPixel ? hoveredPixel.y : 0}
        sendTimeout = {handleSwitchChange}
        primaryId = {primaryId}
      ></Footer>
    </div>
  );
};

export default Canvas;
