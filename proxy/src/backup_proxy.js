const BackendConnection = require('./proxy_classes');
const WebSocket = require("ws");

let healthChecksPaused = false; // flag for pausing health checks

async function checkServerHealth() {

    console.log ("Running health check...");
    return new Promise ( (resolve, reject) => {
        try {
            // connect through websocket to primary proxy server
           const primaryProxyWs = new WebSocket('ws://localhost:3001')
           primaryProxyWs.on('open', function open() {
               console.log('Connected to primary proxy.');

               // send ping to proxy
               primaryProxyWs.send(JSON.stringify({ command: 'ping' }));
           });
           
           // if recieves pong from primary server it is healthy and can continue to use primary
           primaryProxyWs.on('message', (message) => {
               try {
                   const parsedMessage = JSON.parse(message);
                   console.log("Received message from client (primary proxy):", parsedMessage);
                   if (parsedMessage.command === 'pong') {
                       resolve(true);
                   }
           
               } catch (error) {
                   console.error('Error occured while reading message from primary proxy:', error);
                   resolve(false);
               }
           });

           // if unable to connect to primary proxy server, it is unhealthy
           primaryProxyWs.on('error', (error) => {
                console.error('Error occured while trying to connect to websocket of primary proxy server:', error);
                resolve(false);
           });
       } catch (error) {
           console.error('Error while checking server health:', error);
           resolve(false);
       }
    });
 
}


async function checkPrimaryServerHealth() {
    try {
        console.log("health check paused:" , healthChecksPaused);
        if (healthChecksPaused) {
            console.log ('Backup proxy not ready. Skipping health checks...');
            return;
        }

        const isHealthy = await checkServerHealth();
        console.log("Primary proxy is healthy:", isHealthy);

        if (isHealthy) {
            console.log('Primary proxy server is healthy');
        } else {
            healthChecksPaused = true;
            console.log('Primary proxy server is unhealthy. Replacing with backup proxy server...');
           
            // This is the server that clients will connect to
            const clientServer = new WebSocket.Server({ port: 3001 });
            const backendClient = new BackendConnection();

            // TODO: reload page when trying to connect to backup proxy server

            clientServer.on("connection", (clientSocket) => {
                
                console.log("Client connected to backup proxy");

                // stop health checks after connecting to backup as there is no other healthy proxy
                // to be pinging (checking health status of)
                console.log("Stopping health checks"); 
                clearInterval(interval);

                // Proxy messages from client to backend
                clientSocket.on("message", (message) => {
                    try {
                        const parsedMessage = JSON.parse(message);
                        console.log("Received message from client:", parsedMessage);
                        switch (parsedMessage.command) {
                        // Get the current state of the canvas
                        // Used on the initial load for a client
                        case "get_pixels":
                        backendClient.get_canvas(clientSocket)
                        break;
                        // Set a specific pixel
                        case "set_pixel":
                        backendClient.send_ws(JSON.stringify(parsedMessage.payload));
                        break;
                        default:
                        console.log("Invalid command:", parsedMessage.command);
                    }
                    } catch (error) {
                    console.error("Error parsing message:", error);
                    }
                });
            });
        }
    } catch (error) {
        console.error('Error occurred while checking primary server health:', error);
    }
}

console.log ('Initial value of health check paused:', healthChecksPaused);
// Poll the primary proxy server health endpoint at regular intervals
const healthCheckInterval = 5000; // Interval in milliseconds
const interval = setInterval(checkPrimaryServerHealth, healthCheckInterval);
