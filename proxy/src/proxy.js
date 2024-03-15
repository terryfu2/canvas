const WebSocket = require("ws");
const http = require("http");

let backendClient;

// This is the server that clients will connect to
const clientServer = new WebSocket.Server({ port: 3001 });
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
          http
            .get("http://0.0.0.0:8000/canvas", (res) => {
              let data = "";
              res.on("data", (chunk) => {
                data += chunk;
              });
              res.on("end", () => {
                clientSocket.send(data);
              });
            })
            .on("error", (error) => {
              console.error("Error making request to backend:", error.message);
            });
          break;
        // Set a specific pixel
        case "set_pixel":
          backendClient.send(JSON.stringify(parsedMessage.payload));
          break;
        default:
          console.log("Invalid command:", parsedMessage.command);
      }
    } catch (error) {
      console.error("Error parsing message:", error);
    }
  });
});

// This is the connection to the primary replica
// I think that this should probably be replaced by a simpler solution eventually
function connectToBackend() {
  backendClient = new WebSocket("ws://0.0.0.0:8000/ws");

  backendClient.on("open", () => {
    console.log("Connected to backend");
  });

  backendClient.on("close", () => {
    console.log("Connection to backend closed");
    // Attempt to reconnect after 5 seconds
    setTimeout(connectToBackend, 5000);
  });

  backendClient.on("error", (error) => {
    console.error("Error connecting to backend:", error.message);
    // Attempt to reconnect after 5 seconds
    setTimeout(connectToBackend, 5000);
  });

  backendClient.on("message", (message) => {
    try {
      const parsedMessage = JSON.parse(message);
      console.log("Received message from backend:", parsedMessage);
      clientServer.clients.forEach((clientSocket) => {
        clientSocket.send(
          JSON.stringify({
            command: "set_pixel",
            payload: parsedMessage,
          })
        );
      });
    } catch (error) {
      console.error("Error parsing message:", error);
    }
  });
}
connectToBackend();
