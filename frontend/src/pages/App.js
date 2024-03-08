import "../styles/App.scss";

import Canvas from "../components/Canvas";
import { Pixel } from "../objects/Pixel";
import React, { useCallback } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Blocks } from "react-loader-spinner";

const WS_URL = `ws://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}/ws`;

function App() {
  const { sendMessage, lastMessage, readyState } = useWebSocket(WS_URL);

  const pixels = [];

  for (var i = 0; i < 210; i++) {
    for (var j = 0; j < 210; j++) {
      pixels.push(new Pixel(i * 10, j * 10, "white"));
    }
  }

  const sendPixelData = useCallback(
    (x, y, color) =>
      sendMessage(`{"x":${x / 10}, "y":${y / 10}, "colour":${color}}`),
    []
  );

  if (readyState !== ReadyState.OPEN) {
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
        onPixelChange={sendPixelData}
        width={2010}
        height={2010}
        pixels={pixels}
      ></Canvas>
    </div>
  );
}

export default App;
