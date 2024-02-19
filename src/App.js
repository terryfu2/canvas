import Editor from './components/Editor';
import Canvas from './components/Canvas';
import Header from './components/Header';
import { Title } from './components/Title';
import { Footer } from './components/Footer';
import './styles/App.scss';

function App() {
    return (
    <div className="App">
        <Title/>
        <Canvas 
            width={225}
            height={150}
        /> 
        <Footer/>
    </div>
    ); 
}

export default App;