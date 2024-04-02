
const BackendConnection = require('./proxy_classes');
const WebSocket = require("ws");

// This is the server that clients will connect to
const clientServer = new WebSocket.Server({ port: 3001 });

// Manages connections to the backends
const backendClient = new BackendConnection()

clientServer.on("connection", (clientSocket) => {

  console.log("Client connected");

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
        case "ping":
            // receive ping from backup proxy and respond with pong (for health checking purposes)
            clientSocket.send(JSON.stringify({ command: 'pong' }));
            break;
        default:
          console.log("Invalid command:", parsedMessage.command);
      }
    } catch (error) {
      console.error("Error parsing message:", error);
    }
  });
});