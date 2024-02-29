const http = require('http');
const httpProxy = require('http-proxy');

// Create a new proxy server instance
const proxy = httpProxy.createProxyServer({});

// Create HTTP proxy server
const server = http.createServer((req, res) => {
    // Proxy HTTP requests
    // proxy.web(req, res, { target: `http://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}` });
    proxy.web(req, res, { target: `http://127.0.0.1:8000` });
});

// Listen for `upgrade` event (will establish WebSocket connection by upgrading a HTTP request)
server.on('upgrade', (req, socket, head) => {
    // proxy.ws(req, socket, head, { target: `ws://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}` });
    proxy.ws(req, socket, head, { target: `ws://127.0.0.1:8000` });
});

// Handle errors
proxy.on('error', (err, req, res) => {
    res.writeHead(500, {
        'Content-Type': 'text/plain'
    });

    res.end('Something went wrong.');
});

// Start the server lisenting on http port
const PORT = 3001;
server.listen( PORT, () => {
    console.log(`Proxy server listening on port ${PORT}`);
});