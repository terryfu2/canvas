import Canvas from '../components/Canvas';
import { Title } from '../components/common/Title';
import '../styles/App.scss';

function App() {
    return (
    <div className="App">
        <Title/>
        <Canvas 
            width={226}
            height={151}
        /> 
        
    </div>
    ); 
}

export default App;