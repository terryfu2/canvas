import React, { StrictMode } from 'react';
//import ReactDOM from 'react-dom';
import { createRoot } from 'react-dom/client';
import './styles/index.css'
import App from './pages/App';

const container = document.getElementById('root');
const root = createRoot(container);

root.render(
    <StrictMode>
        <App />
    </StrictMode>
);