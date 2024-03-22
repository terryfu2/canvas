import Color from "colorjs.io";
import React, { useEffect, useRef, useState } from "react";
import { MapInteractionCSS } from "react-map-interaction";
import PixelPopUp from "./common/PixelPopUp";
import Footer from "./footer/Footer";

const Canvas = ({ setPixel, width, height, pixels }) => {
  const canvasRef = useRef(null); 

  const [dialogCoordinates, setDialogCoordinates] = useState(null);
  const [hoveredPixel, setHoveredPixel] = useState({ x: 0, y: 0 });
  const [clickedPixel, setClickedPixel] = useState(null);
  const [confirmClicked, setConfirmClicked] = useState(false);
  const [timeoutEnabled, setTimeoutEnabled] = useState(false);



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

  const handleClickPixel = (event) => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    const rect = canvas.getBoundingClientRect();
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

  const handleMouseMove = (event) => {
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

  const handleConfirm = (color) => {

    if(clickedPixel.color == 'white' && color == '#ffffff' || clickedPixel.color == color){
        setDialogCoordinates(null);
        setClickedPixel(null);
        return;
    }
    setConfirmClicked(true);

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    clickedPixel.setColor(color);
    ctx.fillStyle = color; // Set color
    let colorNum = Number(
      `0x${new Color(color).toString({ format: "hex" }).substring(1)}`
    ); // there must be a better way
    //console.log(colorNum);
    ctx.fillRect(clickedPixel.x, clickedPixel.y, 10, 10);
    setPixel(clickedPixel.x, clickedPixel.y, colorNum);

    setDialogCoordinates(null);
    setClickedPixel(null);

    if(timeoutEnabled){
        setTimeout(() => {
            setConfirmClicked(false);
        }, 5000);     
    }
    if(!timeoutEnabled){
        setConfirmClicked(false);
    }
  };

  const handleSwitchChange = (data) => {
    setTimeoutEnabled(!timeoutEnabled);
    console.log(timeoutEnabled);

  };

  return (
    <div>
      <MapInteractionCSS>
        <canvas
          ref={canvasRef}
          width={width}
          height={height}
          style={{
            width: "100%",
            height: "100%",
            cursor: "pointer",
            imageRendering: "pixelated",
          }}
          onClick={handleClickPixel}
          onMouseMove={handleMouseMove}
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
      ></Footer>
    </div>
  );
};

export default Canvas;
