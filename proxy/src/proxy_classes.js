const WebSocket = require("ws");
const http = require("http");

// single backend connection
class BackendInstance {
  constructor(address, port, id, primary, clientServer) {
    console.log(`Connecting to id ${id} at ${address}:${port}`);
    this.address = address;
    this.port = port;
    this.primary = primary;
    this.id = id; // Going to be used for debugging
    this.backoffTime = 5000;
    this.reconnectAttempts = 0;
    this.clientServer = clientServer;

    this.connect();
  }

  onClose() {
    this.primary = false;
    // TODO reimplement this, but have it work
    // Attempt to reconnect after timeout
    // this.reconnectAttempts++;
    setTimeout(() => this.connect(), 5000);
  }

  onMessage(message) {
    try {
      console.log(
        `BACKEND ${this.id}::Received message from backend:`,
        message.toString()
      );
      if (message.toString() === `primary`) {
        // Not the smartest message I know
        console.log(
          `BACKEND ${this.id}::Received message from backend:`,
          message
        );
        this.onPrimaryMessage();
      } else {
        const parsedMessage = JSON.parse(message);
        console.log(
          `BACKEND ${this.id}::Received parsed message from backend:`,
          parsedMessage
        );
        this.onSetPixel(parsedMessage);
      }
    } catch (error) {
      console.error(`BACKEND ${this.id}::Error parsing message:`, error);
    }
  }

  onPrimaryMessage() {
    console.log(`BACKEND ${this.id}::We are connected to the primary`);
    this.primary = true;
    //console.log("send primary to clients");
    this.clientServer.clients.forEach((clientSocket) => {
      clientSocket.send(
        JSON.stringify({
          command: "primary_id",
          payload: this.id,
        })
      );
    });
  }

  onSetPixel(message) {
    let json_message = JSON.stringify({
      command: "set_pixel",
      payload: message,
    });
    this.clientServer.clients.forEach((clientSocket) => {
      clientSocket.send(json_message);
    });
  }

  connect() {
    this.ws_connection = new WebSocket(`ws://${this.address}:${this.port}/ws`);

    this.ws_connection.on("open", () => {
      console.log(`BACKEND ${this.id}::Connected to backend`);
      this.reconnectAttempts = 0;
    });

    this.ws_connection.on("close", () => {
      console.log(`BACKEND ${this.id}::Connection to backend closed`);
      this.onClose();
    });

    this.ws_connection.on("error", (error) => {
      console.error(
        `BACKEND ${this.id}::Error connecting to backend:`,
        error.message
      );
      // Change this to onError
      // this.onClose();
    });

    this.ws_connection.on("message", (message) => {
      this.onMessage(message);
    });
  }

  get_canvas(clientSocket) {
    http
      .get(`http://${this.address}:${this.port}/canvas`, (res) => {
        let data = "";
        res.on("data", (chunk) => {
          data += chunk;
        });
        res.on("end", () => {
          clientSocket.send(data);
        });
      })
      .on("error", (error) => {
        console.error(
          `BACKEND ${id}::Error making request to backend:`,
          error.message
        );
      });
  }

  send_ws(msg) {
    this.ws_connection.send(msg);
  }
}

// Connects to all backends and keeps track of primary
class BackendConnection {
  constructor(clientServer) {
    var fs = require("fs");

    let connection_info = JSON.parse(
      fs.readFileSync("../../process_connections.json", "utf8")
    );

    this.backendInstances = [];

    connection_info["backend"].forEach((connection) => {
      let backendConnection = new BackendInstance(
        connection.public_address,
        connection.public_port,
        connection.id,
        false,
        clientServer
      );
      this.backendInstances.push(backendConnection);
    });
    this.backendInstances[0].primary = true;
  }

  find_primary() {
    return this.backendInstances.find((instance) => instance.primary);
  }

  get_canvas(clientSocket) {
    let primary = this.find_primary();
    if (primary == undefined) {
      console.error(`No primary found!`);
      return;
    } else {
      console.log("Primary id: " + primary.id);
      //console.log("send primary to clinets");
      clientSocket.send(
        JSON.stringify({
          command: "primary_id",
          payload: primary.id,
        })
      );
    }
    primary.get_canvas(clientSocket);
  }

  send_ws(msg) {
    let primary = this.find_primary();
    if (primary == undefined) {
      console.error(`No primary found!`);
      return;
    }
    primary.send_ws(msg);
  }
}

module.exports = BackendConnection;
