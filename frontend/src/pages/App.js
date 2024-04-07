import "../styles/App.scss";

import Canvas from "../components/Canvas";
import { Pixel } from "../objects/Pixel";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Blocks } from "react-loader-spinner";
import React, { useCallback, useEffect, useState } from "react";
import Alert from '@mui/material/Alert';
import Collapse from '@mui/material/Collapse';

const WS_URL = `ws://${process.env.REACT_APP_HTTP_HOST}:${process.env.REACT_APP_HTTP_PORT}/ws`;

function App() {
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(WS_URL);
  const [pixels, setPixels] = useState();
  const [openSuccess,setOpenSuccess] = useState(false);
  const [openError,setOpenError] = useState(false);
  const [primaryId,setPrimaryId] = useState(null);

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

  const isError = () => {
    setTimeout(() => {
      setOpenSuccess(prevOpenSuccess => {
        //console.log(prevOpenSuccess);
        if (!prevOpenSuccess) {
          setOpenError(true);
          setTimeout(() => {
            setOpenError(false);
          }, 2000);
        }
        return prevOpenSuccess; 
      });
    }, 1000);
  };

  useEffect(() => {
    getPixels();
  }, []);

  useEffect(() => {
    if (lastJsonMessage !== null) {
     //console.log(lastJsonMessage);
      switch (lastJsonMessage.command) {
        case "get_pixels":
          const newPixels = [];
          for (var i = 0; i < 510; i++) {
            for (var j = 0; j < 510; j++) {
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
          setOpenSuccess(true);
          setOpenSuccess(prevState => {
            //console.log("asdf" + prevState);
            setTimeout(() => {
              setOpenSuccess(false);
            }, 2000);
            return true;
          });
          //console.log("asdf"+openSuccess);
          setTimeout(() => {
            setOpenSuccess(false);
            }, 2000); 
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
        case "primary_id":
            //console.log("app" +lastJsonMessage.payload);
            //setPrimaryId(lastJsonMessage.payload);
            setPrimaryId(prevPrimaryId => lastJsonMessage.payload);
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
      <Canvas setPixel={setPixel} isError = {isError} pixels={pixels} width={5010} height={5010} primary={primaryId} />

      <Collapse in={openSuccess}>
        <Alert
            style={{
                position: 'absolute',
                bottom: '57%', 
                left: '50%', 
                transform: 'translateX(-50%)',
                zIndex: 10000 
            }}
            variant="filled"
            severity="success"
            >
            pixel successfully updated!
            </Alert>
        </Collapse>
        <Collapse in={openError}>
        <Alert
            style={{
                position: 'absolute',
                bottom: '57%', 
                left: '50%', 
                transform: 'translateX(-50%)',
                zIndex: 10000 
            }}
            variant="filled"
            severity="error"
            >
            pixel not updated!
            </Alert>
        </Collapse>
    </div>
  );
}

export default App;
