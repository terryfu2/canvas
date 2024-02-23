import '../styles/App.scss';

import Canvas from '../components/Canvas';
import { Pixel } from '../objects/Pixel';
import { Title } from '../components/common/Title';

function App() {
    
    // temp logic to show data in frontend 
    const pixels = [
        new Pixel(0, 0, 'red'),
        new Pixel(10, 10, 'blue'),
        new Pixel(20, 20, 'green')
    ];
    
    for(var i = 0;i<1000;i++){
        for(var j = 0;j<1000;j++){

            

            pixels.push(new Pixel(i*10,j*10,'white'));
        }
    }

    console.log(pixels);
    
    return (
        <div className="App">
            <Title></Title>
            <Canvas width={window.innerWidth} height={window.innerHeight} pixels={pixels}></Canvas>
        </div>
    ); 
}

export default App;