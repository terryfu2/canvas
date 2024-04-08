import "../styles/App.scss";

import Canvas from "../components/Canvas";
import { Pixel } from "../objects/Pixel";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Blocks } from "react-loader-spinner";
import React, { useCallback, useEffect, useState } from "react";
import Alert from "@mui/material/Alert";
import Collapse from "@mui/material/Collapse";

const WS_URL = `ws://${process.env.REACT_APP_HTTP_HOST}:${process.env.REACT_APP_HTTP_PORT}/ws`;
const MAX_RETRY_ATTEMPTS = 100; // Maximum number of retry attempts
const RETRY_INTERVAL = 1000; // Retry interval in milliseconds

function App() {
  //const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(WS_URL);
  const [pixels, setPixels] = useState();
  const [openSuccess, setOpenSuccess] = useState(false);
  const [openError, setOpenError] = useState(false);
  const [primaryId, setPrimaryId] = useState(null);
  const [retryAttempts, setRetryAttempts] = useState(0);

  //auto retry coinnection to proxy if not loading, for when proxy crashes
  const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(
    WS_URL,
    {
      onOpen: () => {
        setRetryAttempts(0); // Reset retry attempts on successful connection
      },
    }
  );

  const retryConnection = () => {
    if (retryAttempts < MAX_RETRY_ATTEMPTS) {
      setTimeout(() => {
        setRetryAttempts((prevAttempts) => prevAttempts + 1);
      }, RETRY_INTERVAL);
    }
  };

  useEffect(() => {
    if (readyState === ReadyState.CLOSED) {
      retryConnection();
      window.location.reload();
    }
  }, [readyState, retryAttempts]);

  //message to proxy
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
  //error popup
  const isError = () => {
    setTimeout(() => {
      setOpenSuccess((prevOpenSuccess) => {
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
  //received json message, parse accordingly
  useEffect(() => {
    if (lastJsonMessage !== null) {
      switch (lastJsonMessage.command) {
        case "get_pixels":
          const newPixels = [];
          for (var i = 0; i < 510; i++) {
            for (var j = 0; j < 510; j++) {
              newPixels.push(new Pixel(i * 10, j * 10, "white"));
            }
          }
          for (let pixel of lastJsonMessage.payload) {
            let new_color = pixel.colour.toString(16);
            while (new_color.length < 6) {
              new_color = "0" + new_color;
            }
            newPixels.push(
              new Pixel(pixel.x * 10, pixel.y * 10, `#${new_color}`)
            );
          }
          setPixels(newPixels);
          break;
        case "set_pixel":
          const payload = lastJsonMessage.payload;
          setOpenSuccess(true);
          setOpenSuccess((prevState) => {
            //console.log("asdf" + prevState);
            setTimeout(() => {
              setOpenSuccess(false);
            }, 2000);
            return true;
          });
          setPixels((pixels) =>
            pixels.map((pixel) => {
              if (pixel.x / 10 === payload.x && pixel.y / 10 === payload.y) {
                let new_color = payload.colour.toString(16);
                while (new_color.length < 6) {
                  new_color = "0" + new_color;
                }
                console.log(`Setting to ${pixel.x}, ${pixel.y}, #${new_color}`);
                return new Pixel(pixel.x, pixel.y, `#${new_color}`);
              } else {
                return pixel;
              }
            })
          );
          break;
        case "primary_id":
          //console.log("app" +lastJsonMessage.payload);
          //setPrimaryId(lastJsonMessage.payload);
          setPrimaryId((prevPrimaryId) => lastJsonMessage.payload);
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
      <Canvas
        setPixel={setPixel}
        isError={isError}
        pixels={pixels}
        width={5010}
        height={5010}
        primary={primaryId}
      />

      <Collapse in={openSuccess}>
        <Alert
          style={{
            position: "absolute",
            bottom: "57%",
            left: "50%",
            transform: "translateX(-50%)",
            zIndex: 10000,
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
            position: "absolute",
            bottom: "57%",
            left: "50%",
            transform: "translateX(-50%)",
            zIndex: 10000,
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
