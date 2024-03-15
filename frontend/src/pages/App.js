import "../styles/App.scss";

import Canvas from "../components/Canvas";
import { Pixel } from "../objects/Pixel";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Blocks } from "react-loader-spinner";
import React, { useCallback, useEffect, useState } from "react";

const WS_URL = `ws://${process.env.REACT_APP_HTTP_HOST}:${process.env.REACT_APP_HTTP_PORT}/ws`;

function App() {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(WS_URL);
  const [pixels, setPixels] = useState();

  const getPixels = useCallback(
    () =>
      sendJsonMessage({
        command: "get_pixels",
      }),
    [sendJsonMessage]
  );

  const setPixel = useCallback(
    (x, y, color) =>
      sendJsonMessage({
        command: "set_pixel",
        payload: {
          x: x / 10,
          y: y / 10,
          colour: color,
        },
      }),
    [sendJsonMessage]
  );

  useEffect(() => {
    getPixels();
  }, []);

  useEffect(() => {
    if (lastJsonMessage !== null) {
      switch (lastJsonMessage.command) {
        case "get_pixels":
          const newPixels = [];
          for (var i = 0; i < 210; i++) {
            for (var j = 0; j < 210; j++) {
              newPixels.push(new Pixel(i * 10, j * 10, "white"));
            }
          }
          for (let pixel of lastJsonMessage.payload) {
            newPixels.push(
              new Pixel(
                pixel.x * 10,
                pixel.y * 10,
                `#${pixel.colour.toString(16)}`
              )
            );
          }
          setPixels(newPixels);
          break;
        case "set_pixel":
          const payload = lastJsonMessage.payload;
          setPixels(
            pixels.map((pixel) => {
              if (pixel.x / 10 === payload.x && pixel.y / 10 === payload.y) {
                return new Pixel(
                  pixel.x,
                  pixel.y,
                  `#${payload.colour.toString(16)}`
                );
              } else {
                return pixel;
              }
            })
          );
          break;
        default:
          console.error("Received invalid message:", lastJsonMessage);
      }
    }
  }, [lastJsonMessage]);

  if (readyState !== ReadyState.OPEN || !pixels) {
    return (
      <div className="loading">
        <Blocks
          height="80"
          width="80"
          color="#4fa94d"
          ariaLabel="blocks-loading"
          wrapperStyle={{}}
          wrapperClass="blocks-wrapper"
          visible={true}
        />
      </div>
    );
  }

  return (
    <div className="App">
      <Canvas setPixel={setPixel} pixels={pixels} width={2010} height={2010} />
    </div>
  );
}

export default App;
