import '../styles/App.scss';

import Canvas from '../components/Canvas';
import { Pixel } from '../objects/Pixel';
import React, {useCallback} from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';

const WS_URL = `ws://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}/ws`;

function App() {
    const { sendMessage, lastMessage, readyState } = useWebSocket(WS_URL);
    
    const pixels = [];
    
    for(var i = 0;i<1000;i++){
        for(var j = 0;j<1000;j++){

            pixels.push(new Pixel(i*10,j*10,'white'));
        }
    }
    
    const sendPixelData = useCallback((x,y,color) => sendMessage(`{"x":${x/10}, "y":${y/10}, "colour":${color}}`), []);
    
    return (
        <div className="App" >

            <Canvas onPixelChange={sendPixelData} width={1980} height={1020} pixels={pixels}></Canvas>
        </div>
    ); 
}

export default App;