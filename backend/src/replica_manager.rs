//! A multi-room chat server.
use crate::pixel::Pixel;
use crate::Msg;
use deadpool_postgres::Pool;
use futures::select;
use futures::FutureExt;
use rand::{thread_rng, Rng as _};
use serde_json;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::pin;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot};

/// A command received by the Replica
#[derive(Debug)]
pub enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<usize>,
    },

    Message {
        msg: Msg,
        res_tx: oneshot::Sender<()>,
    },

    Disconnect {
        conn: usize,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ConnectionInfo {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ReplicaInfo {
    pub id: u16,
    pub public_address: String,
    pub public_port: u16,
    pub address: String,
    pub socket_port: u16,
    pub active: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ConnectionInfoDict {
    pub frontend: ConnectionInfo,
    pub proxy: ConnectionInfo,
    pub backend: Vec<ReplicaInfo>,
}

impl ConnectionInfoDict {
    fn get_socket_addr(backend: &Vec<ReplicaInfo>, id: u16) -> SocketAddrV4 {
        let replica_info: &ReplicaInfo = &backend.iter().find(|r| r.id == id).unwrap();
        let addr: Ipv4Addr = replica_info.address.parse::<Ipv4Addr>().unwrap();
        SocketAddrV4::new(addr, replica_info.socket_port)
    }

    fn get_successor_id(backend: &Vec<ReplicaInfo>, id: u16) -> u16 {
        let index = backend.iter().position(|r| r.id == id).unwrap();

        // Create an iterator that starts from the specified index and cycles back to the beginning
        let iter = backend.iter().cycle().skip(index + 1);

        // Find the first element with active field set to true
        for item in iter {
            if item.active {
                return item.id;
            }
        }

        id // No active element found
    }

    fn get_predecessor_id(backend: &Vec<ReplicaInfo>, id: u16) -> u16 {
        let index = backend.iter().rev().position(|r| r.id == id).unwrap();

        // Create an iterator that starts from the specified index and cycles back to the beginning, but in reverse
        let iter = backend.iter().rev().cycle().skip(index + 1);

        // Find the first element with active field set to true
        for item in iter {
            if item.active {
                return item.id;
            }
        }

        id // No active element found
    }
    fn get_own_info(backend: &Vec<ReplicaInfo>, id: u16) -> &ReplicaInfo {
        backend.iter().find(|r| r.id == id).unwrap()
    }

    fn get_own_info_str(backend: &Vec<ReplicaInfo>, id: u16) -> String {
        serde_json::to_string(backend.iter().find(|r| r.id == id).unwrap()).unwrap()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NewConMessage {
    from: ReplicaInfo,
    effecting: ReplicaInfo,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncMessage {
    pixels: Vec<Pixel>,
    conn: ConnectionInfoDict,
    leader: u16,
    predecessor_id: u16,
}

// TODO calc max size or find it experimentally
const REPLICA_BUFFER_SIZE: usize = 10240000;
const SMALL_REPLICA_BUFFER_SIZE: usize = 10000;

fn connections_file() -> String {
    std::env::var("CONNECTIONS_FILE").unwrap_or_else(|_| "../../process_connections.json".into())
}

fn proc_id() -> u16 {
    std::env::var("ID")
        .unwrap_or_else(|_| "0".into())
        .parse::<u16>()
        .unwrap_or_else(|_| 0)
}

fn is_debug_enabled() -> bool {
    match std::env::var("DEBUG") {
        Ok(val) => {
            if val == "1" || val.to_lowercase() == "true" {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
/// Manages the messages to and from replicas.
///
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct ReplicaManager {
    /// Map of connection IDs to their message receivers.
    sessions: HashMap<usize, mpsc::UnboundedSender<Msg>>,

    is_primary: bool,

    /// Process id
    id: u16,

    /// Postgres db_connection
    db: Pool,

    successor_stream: Option<TcpStream>,

    election_running: bool,

    // We can add this back later
    // predecessor_stream: Option<TcpStream>
    connections_info: ConnectionInfoDict,
    predecessor_id: u16,
    successor_id: u16,

    leader_id: u16,

    expected_queue: Arc<Mutex<VecDeque<String>>>,

    connected: bool,

    sent_sync: bool,
}

impl ReplicaManager {
    pub fn new(
        is_primary: bool,
        db: Pool,
        cmd_tx: mpsc::UnboundedSender<Command>,
    ) -> (Self, ReplicaHandle) {
        let id = proc_id();
        log::info!("Proc id is {}", id);
        // Open the file in read-only mode with buffer.
        let file = File::open(connections_file()).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file
        let connections_info: ConnectionInfoDict = serde_json::from_reader(reader).unwrap();

        let successor_id = ConnectionInfoDict::get_successor_id(&connections_info.backend, id);
        let predecessor_id = ConnectionInfoDict::get_predecessor_id(&connections_info.backend, id);

        let expected_queue = Arc::new(Mutex::new(VecDeque::new()));

        // Leader starts as first instance of backend list
        let leader_id = connections_info.backend[0].id;
        log::info!("Successor id {}", successor_id);
        log::info!("Predecessor id {}", predecessor_id);
        log::info!("Leader id {}", leader_id);
        (
            Self {
                sessions: HashMap::new(),
                is_primary,
                id,
                db,
                successor_stream: None,
                // predecessor_stream: None,
                election_running: false,
                connections_info,
                predecessor_id: predecessor_id,
                successor_id: successor_id,
                leader_id,
                expected_queue,
                connected: false,
                sent_sync: false
            },
            ReplicaHandle { cmd_tx },
        )
    }

    /// Parse msg and send to appropriate manager
    pub async fn handle_replica_msg(&mut self, msg: String) -> io::Result<()> {
        let msg: String = msg.trim().to_string();

        if msg.starts_with('/') {
            let mut cmd_args = msg.splitn(2, ' ');

            // unwrap: we have guaranteed non-zero string length already
            match cmd_args.next().unwrap() {
                "/all_pixels" => match cmd_args.next() {
                    Some(pixels) => self.handle_all_pixels_msg(pixels.to_string()).await,
                    None => {
                        log::info!("No pixels provided to all pixels update");
                    }
                },
                // '/election election/leader id'
                "/election" => match cmd_args.next() {
                    Some(id) => self.handle_election_msg(id.to_string()).await?,
                    None => {
                        log::info!("No ID provided for election");
                    }
                },

                "/disconnect" => match cmd_args.next() {
                    Some(id) => self.handle_disconnect_msg(id.to_string()).await?,
                    None => {
                        log::info!("No ID provided for disconnect");
                    }
                },
                "/new_connection" => match cmd_args.next() {
                    Some(message) => self.handle_new_connection_msg(message.to_string()).await?,
                    None => {
                        log::info!("No ID provided for disconnect");
                    }
                },
                "/sync" => match cmd_args.next() {
                    Some(message) => self.handle_sync_msg(message.to_string()).await?,
                    None => {
                        log::info!("No ID provided for disconnect");
                    }
                },

                _ => {
                    log::info!("Unknown command {}", msg);
                }
            }
        } else {
            self.handle_pixel_msg(msg).await;
        }
        return Ok(());
    }

    /// Normal pixel update, add it to db
    /// Really these should return errors too, but to lazy to box
    pub async fn handle_pixel_msg(&mut self, msg: String) {
        // For testing consistency
        if is_debug_enabled() {
            log::info!("Pixel update received: {}", msg);
            log::info!("DEBUG is enabled so we are not sending to successor");
            return;
        }

        if self.is_primary {
            let expected = self.expected_queue.lock().unwrap().pop_front();
            match expected {
                None => log::warn!("Received unexpected pixel message: {}", msg),
                Some(expected) => {
                    if expected == msg {
                        log::info!("Validated expected pixel message: {}", msg);
                        let db = self.db.get().await.unwrap();
                        Pixel::insert(db, msg.clone()).await.unwrap();
                        self.send_replicated_to_ws(msg).await;
                    } else {
                        log::info!("Invalid pixel message: {}, expected: {}", msg, expected);
                    }
                }
            }
            return;
        }
        log::info!("Pixel update received: {}", msg);
        let db = self.db.get().await.unwrap();
        Pixel::insert(db, msg.clone()).await.unwrap();

        if !self.is_primary {
            log::info!("Sent message to successor: {}", msg);
            self.send_successor(msg.as_bytes()).await.unwrap();
        } else {
            log::info!("Ignored message: {}", msg);
        }
    }

    /// Clear and set the entire database to list of pixels provided
    /// Really these should return errors too, but to lazy to box
    pub async fn handle_all_pixels_msg(&mut self, msg: String) {
        if self.is_primary {
            // We already updated our database, do nothing
            return;
        }
        log::info!("All pixels update received");
        // log::info!("All pixels update received: {}", msg);
        let db = self.db.get().await.unwrap();
        Pixel::update_all(db, &msg).await.unwrap();

        if !self.is_primary {
            log::info!("Sent all_pixels message to successor");
            // log::info!("Sent message to successor: {}", msg);
            let pixels_str = format!("/all_pixels {}", msg);
            self.send_successor(pixels_str.as_bytes()).await.unwrap();
        } else {
            log::info!("Ignored all pixels update");
        }
    }

    pub async fn initiate_election(&mut self) -> io::Result<()> {
        log::info!("Election started");
        self.election_running = true;
        let election_message = format!("/election election {}", self.id);
        self.send_successor(election_message.as_bytes()).await
    }

    /// We can do elections here
    pub async fn handle_election_msg(&mut self, msg: String) -> io::Result<()> {
        log::info!("Election message received: {}", msg);
        let mut cmd_args = msg.splitn(2, ' ');
        let election_type = match cmd_args.next() {
            Some(msg) => msg,
            None => {
                log::error!("Election error No election/leader specifier");
                "no_election"
            }
        };
        let id = match cmd_args.next() {
            Some(id) => match id.parse::<u16>() {
                Ok(val) => val,
                Err(e) => {
                    log::error!("Election error: {}", e);
                    0
                }
            },
            None => {
                log::error!("Election error No id specified");
                0
            }
        };

        if election_type == "leader" {
            self.election_running = false;
            log::info!("New leader elected: {}", id);
            self.leader_id = id;
            if id != self.id {
                self.is_primary = false;
                let election_message = format!("/election leader {}", id);
                log::info!("Election sending: {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            } else {
                log::info!("Election we are the primary");
                self.is_primary = true;
                // let the websocket sessions know
                self.send_primary_to_ws().await;
            }
        } else if election_type == "election" {
            if id > self.id {
                let election_message = format!("/election election {}", id);
                log::info!("Election sending: {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
            if id < self.id && !self.election_running {
                self.election_running = true;
                let election_message = format!("/election election {}", self.id);
                log::info!("Election sending: {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
            if id == self.id {
                let election_message = format!("/election leader {}", self.id);
                log::info!("Election sending: {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
        } else {
            log::error!(
                "Election error unrecognized election type: {}",
                election_type
            );
        }
        return Ok(());
    }

    pub async fn handle_sync_msg(&mut self, msg: String) -> io::Result<()> {
        log::info!("Got sync");
        let mut sync: SyncMessage = serde_json::from_str(&msg).unwrap();
        self.predecessor_id = sync.predecessor_id;
        if self.is_primary {
            // We already updated our database, do nothing
            if !self.sent_sync {
                self.connections_info = sync.conn.clone();
                log::info!(
                    "New dict {}",
                    serde_json::to_string(&self.connections_info).unwrap()
                );
                // self.predecessor_id = ConnectionInfoDict::get_predecessor_id(&self.connections_info.backend, self.id);
                log::info!("Successor id {}", self.successor_id);
                log::info!("Predecessor id {}", self.predecessor_id);
                self.leader_id = sync.leader;
                log::info!("Our leader is {}", self.leader_id);
                self.send_initial_sync().await?;
            } else {
                self.sent_sync = false;
                log::info!("Successor id {}", self.successor_id);
                log::info!("Predecessor id {}", self.predecessor_id);
                log::info!("Our leader is {}", self.leader_id);
            }

            
            return Ok(());
        }
        log::info!("All pixels update received");
        // log::info!("All pixels update received: {}", msg);
        let db = self.db.get().await.unwrap();
        Pixel::update_all_vec(db, &sync.pixels).await.unwrap();

        self.connections_info = sync.conn.clone();
        log::info!(
            "New dict {}",
            serde_json::to_string(&self.connections_info).unwrap()
        );
        log::info!("Successor id {}", self.successor_id);
        log::info!("Predecessor id {}", self.predecessor_id);
        self.leader_id = sync.leader;
        log::info!("Our leader is {}", self.leader_id);

        sync.predecessor_id = self.id;
        let sync_str = serde_json::to_string(&sync).unwrap();
        let new_str = format!("/sync {}", sync_str);
        self.send_successor(new_str.as_bytes()).await?;
        Ok(())
    }

    /// Received a new connection message.
    pub async fn handle_new_connection_msg(&mut self, msg: String) -> io::Result<()> {
        let new_conn_message = serde_json::from_str::<NewConMessage>(&msg).unwrap();
        let from_info = new_conn_message.from;
        let effected_info = new_conn_message.effecting;
        if effected_info.id != self.successor_id {
            // We don't care, just forward
            let new_con_str = format!("/new_connection {}", msg);
            return self.send_successor(new_con_str.as_bytes()).await;
        }

        let addr: Ipv4Addr = from_info.address.parse::<Ipv4Addr>().unwrap();
        log::info!("Trying to connect to {}", addr);
        match TcpStream::connect(SocketAddrV4::new(addr, from_info.socket_port)).await {
            Ok(stream) => {
                log::info!("Connected to {}", addr);
                self.successor_stream = Some(stream);
                self.successor_id = from_info.id;
                self.connected = true;
            }
            Err(e) => log::error!("Couldn't connect to {} because {}", from_info.id, e),
        }

        Ok(())
    }

    /// Received a disconnect message. If its our successor get a new connection
    pub async fn handle_disconnect_msg(&mut self, msg: String) -> io::Result<()> {
        log::info!("Disconnect message received: {}", msg);

        // Get id that disconnected
        let id = match msg.parse::<u16>() {
            Ok(val) => val,
            Err(e) => {
                log::error!(
                    "Disconnect parse error: {} using value :{}",
                    e,
                    self.successor_id
                );
                self.successor_id
            }
        };

        self.connections_info
            .backend
            .retain(|backend| backend.id != id);
        log::info!(
            "New connection dict {}",
            serde_json::to_string(&self.connections_info).unwrap()
        );
        log::info!("Successor id {}", self.successor_id);
        log::info!("Predecessor id {}", self.predecessor_id);
        log::info!("Leader id {}", self.leader_id);

        if id == self.successor_id {
            let new_id =
                ConnectionInfoDict::get_successor_id(&self.connections_info.backend, self.id);
            log::info!(
                "Found new successor attempting to establish connection. id: {}",
                new_id
            );
            self.successor_id = new_id;
            log::info!("New successor id: {}", self.predecessor_id);
            self.successor_stream = Some(
                TcpStream::connect(ConnectionInfoDict::get_socket_addr(
                    &self.connections_info.backend,
                    new_id,
                ))
                .await?,
            );
            log::info!("Connected");

            // Either this one or the other one needs to start an election
            // I am not sure whats better
            if id == self.leader_id {
                self.initiate_election().await?;
            }
            return Ok(());
        } else {
            // Forward
            let disconnect_msg = format!("/disconnect {}", id);
            self.send_successor(disconnect_msg.as_bytes()).await
        }
    }

    /// Send a message to the successor
    pub async fn send_successor(&mut self, msg: &[u8]) -> io::Result<()> {
        match self.successor_stream.as_mut() {
            Some(successor_stream) => successor_stream.write_all(msg).await,
            None => {
                log::error!("Attempted to successor write with no connection");
                // Maybe could recover and not panic here
                panic!("Attempted to successor write with no connection");
            }
        }
    }

    /// If the predecessor disconnects we need to notify the other replicas and start an election
    /// Return false if it is no longer possible to recover replica manager process
    pub async fn handle_predecessor_disconnect(
        &mut self,
        cmd_rx: &mut UnboundedReceiver<Command>,
        listener: &TcpListener,
    ) -> io::Result<TcpStream> {
        log::info!("Attempting to connect to new predecessor");
        let id = self.predecessor_id;
        self.connections_info
            .backend
            .retain(|backend| backend.id != id);
        log::info!(
            "New connection dict {}",
            serde_json::to_string(&self.connections_info).unwrap()
        );
        log::info!("Successor id {}", self.successor_id);
        log::info!("Predecessor id {}", self.predecessor_id);
        log::info!("Leader id {}", self.leader_id);

        if self.connections_info.backend.len() == 1 {
            log::info!("We are only backend left");
            self.send_primary_to_ws().await;
            return self.event_loop_until_connect(cmd_rx, listener).await;
        }

        let msg = format!("/disconnect {}", self.predecessor_id);
        log::info!("Sending {} to {}", msg, self.successor_id);
        self.send_successor(msg.as_bytes()).await?;

        let (predecessor_stream, _) = listener.accept().await?;
        self.predecessor_id = ConnectionInfoDict::get_predecessor_id(&self.connections_info.backend, self.id);
        return Ok(predecessor_stream);
    }

    /// Let all ws sessions know that we are the new primary so they can forward that to their proxies
    async fn send_primary_to_ws(&self) {
        let msg = format!("primary");

        for (id, session) in &self.sessions {
            log::info!("Sending primary to session {}", id);
            let _ = session.send(msg.clone());
        }
    }

    /// Let all ws sessions know that the message was successfully applied to all replicas
    async fn send_replicated_to_ws(&self, msg: String) {
        let msg = format!("replicated: {}", msg);

        for (id, session) in &self.sessions {
            log::info!("Sending replicated to session {}", id);
            print!("Sending replicated to session {}", id);
            let _ = session.send(msg.clone());
        }
    }

    /// Register new session and assign unique ID to this session. This is to talk to the other thread
    async fn register_session(&mut self, tx: mpsc::UnboundedSender<Msg>) -> usize {
        // register session with random connection ID
        let id = thread_rng().gen::<usize>();
        log::info!("Registering session {}", id);

        self.sessions.insert(id, tx);

        // send id back
        id
    }

    /// Unregister a session and remove from map
    async fn unregister_session(&mut self, conn_id: usize) {
        log::info!("Unregistering session {}", conn_id);
        self.sessions.remove(&conn_id);
    }

    async fn handle_command(&mut self, cmd: Command) -> io::Result<()> {
        match cmd {
            Command::Connect { conn_tx, res_tx } => {
                let conn_id = self.register_session(conn_tx).await;
                let _ = res_tx.send(conn_id);
                if self.is_primary {
                    self.send_primary_to_ws().await;
                }
            }
            Command::Message { msg, res_tx } => {
                log::info!("Message received: {}", msg);
                if !self.connected {
                    log::info!("Only replica, ignoring message");
                    let _ = res_tx.send(());
                    self.send_replicated_to_ws(msg).await;
                    return Ok(());
                }

                if self.is_primary {
                    // Ensure that the message has been fully replicated
                    // We do this by adding the message to an expected message queue
                    // 5 seconds later we check if the expected message queue no longer contains
                    // that message
                    self.expected_queue.lock().unwrap().push_back(msg.clone());
                    log::info!("Added message to expected message queue");

                    // If you have the displeasure of having to read the following 20 lines, I apologize in advance
                    let msg_clone = msg.clone();
                    let queue_clone = Arc::clone(&self.expected_queue);
                    let sessions_clone = self.sessions.clone();
                    thread::spawn(move || {
                        // Wait for 5 seconds
                        thread::sleep(Duration::from_secs(5));

                        // Check if the first item in the queue has changed
                        let mut queue = queue_clone.lock().unwrap();
                        if let Some(expected) = queue.front() {
                            if expected.as_str() == msg_clone {
                                log::info!("Expected message was not received after 5 seconds");
                                queue.pop_front();

                                let ws_msg = format!("unreplicated: {}", msg_clone);
                                for (id, session) in sessions_clone {
                                    log::info!("Sending unreplicated to session {}", id);
                                    let _ = session.send(ws_msg.clone());
                                }
                            }
                        }
                    });
                }

                self.send_successor(msg.as_bytes()).await?;
                let _ = res_tx.send(());
            }
            Command::Disconnect { conn } => {
                self.unregister_session(conn).await;
            }
        }

        return Ok(());
    }

    pub async fn handle_socket(&mut self, predecessor_stream: &TcpStream) -> io::Result<()> {
        let mut predecessor_buf = vec![0; REPLICA_BUFFER_SIZE];
        match predecessor_stream.try_read(&mut predecessor_buf) {
            Ok(n) => {
                // If the predecessor_stream's proc crashes we get some issues
                if n == 0 {
                    log::error!("Err received size 0 likely predecessor crashed");
                    return Err(io::ErrorKind::WriteZero.into());
                }

                predecessor_buf.truncate(n);
                let predecessor_msg = match str::from_utf8(&predecessor_buf) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                if n < 10000 {
                    log::info!("Received message from socket: {}", predecessor_msg);
                } else {
                    log::info!("Received long message from socket");
                }

                self.handle_replica_msg(predecessor_msg.to_string()).await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // This gets called after every update idk why
                log::error!("Err would block (Don't worry about this I think) {}", e);
            }
            Err(e) => {
                log::error!("Err try_read {}", e);
                return Err(e.into());
            }
        }
        return Ok(());
    }

    pub async fn handle_accepted_stream(
        &mut self,
        stream: &TcpStream,
        alone: bool,
    ) -> io::Result<()> {
        log::info!("Accepting connection");
        let mut predecessor_buf = vec![0; SMALL_REPLICA_BUFFER_SIZE];
        stream.readable().await?;
        let n = stream.try_read(&mut predecessor_buf)?;
        predecessor_buf.truncate(n);
        let conn_info = match str::from_utf8(&predecessor_buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
        .trim();
        log::info!("Received connection from {}", conn_info);
        let new_conn_info = serde_json::from_str::<ReplicaInfo>(&conn_info).unwrap();
        self.connections_info.backend.push(new_conn_info.clone());
        self.predecessor_id = new_conn_info.id;
        if alone {
            // If only replica connect ourselves
            // Parse the IP address
            let ip: Ipv4Addr = new_conn_info
                .address
                .parse()
                .expect("Failed to parse IP address");

            // Create a SocketAddrV4 from the parsed IP address and port number
            let socket_addr_v4 = SocketAddrV4::new(ip, new_conn_info.socket_port);
            log::info!("Connecting to {}", socket_addr_v4);
            self.successor_stream = Some(TcpStream::connect(socket_addr_v4).await?);
            self.successor_id = new_conn_info.id;
            self.connected = true;
            self.send_initial_sync().await?;
            return Ok(());
        }
        let new_conn_message: NewConMessage = NewConMessage {
            from: new_conn_info,
            effecting: ConnectionInfoDict::get_own_info(&self.connections_info.backend, self.id)
                .clone(),
        };
        let new_con_str = format!(
            "/new_connection {}",
            serde_json::to_string(&new_conn_message).unwrap()
        );
        self.send_successor(new_con_str.as_bytes()).await?;
        self.send_initial_sync().await?;
        return Ok(());
    }

    /// New replica was added. Lets sync all again
    pub async fn send_initial_sync(&mut self) -> io::Result<()> {
        self.sent_sync = true;
        let db = self.db.get().await.unwrap();
        let sync = SyncMessage {
            pixels: Pixel::all(&**db).await.unwrap(),
            conn: self.connections_info.clone(),
            leader: self.leader_id,
            predecessor_id: self.id,
        };
        let mut sync_str = serde_json::to_string(&sync).unwrap();

        sync_str = format!("/sync {}", sync_str);
        let sync_bytes = sync_str.as_bytes();
        log::info!("Sending {} bytes", sync_bytes.len());
        self.send_successor(sync_bytes).await?;

        Ok(())
    }

    /// Try to connect to another replica
    pub async fn try_connect(&mut self) -> bool {
        let mut to_return = false;
        for backend in self.connections_info.backend.iter_mut() {
            if backend.id == self.id {
                continue;
            }
            if backend.active {
                let addr: Ipv4Addr = match backend.address.parse() {
                    Ok(addr) => addr,
                    Err(_) => {
                        log::error!("Invalid address: {}", backend.address);
                        backend.active = false;
                        continue;
                    }
                };
                log::info!("Trying to connect to {}", addr);

                match tokio::time::timeout(
                    Duration::from_secs(2),
                    TcpStream::connect(SocketAddrV4::new(addr, backend.socket_port)),
                )
                .await
                {
                    Ok(res) => match res {
                        Ok(stream) => {
                            log::info!("Connected to {}", addr);
                            self.successor_stream = Some(stream);
                            self.successor_id = backend.id;
                            self.connected = true;
                            self.is_primary = false;
                            to_return = true;
                            break;
                        }
                        Err(e) => {
                            log::error!("Couldn't connect to {} because {}", backend.id, e);
                            backend.active = false;
                        }
                    },
                    Err(e) => {
                        log::error!("Timeout {} {}", backend.id, e);
                        backend.active = false;
                    }
                }
            }
        }
        self.connections_info
            .backend
            .retain(|backend| backend.active);
        log::info!(
            "New connection dict {}",
            serde_json::to_string(&self.connections_info).unwrap()
        );
        log::info!("Successor id {}", self.successor_id);
        log::info!("Predecessor id {}", self.predecessor_id);
        log::info!("Leader id {}", self.leader_id);
        
        to_return
    }

    /// Function to run until the initial connection occurs
    pub async fn event_loop_until_connect(
        &mut self,
        cmd_rx: &mut UnboundedReceiver<Command>,
        listener: &TcpListener,
    ) -> io::Result<TcpStream> {
        log::info!("Currently only replica running single event loop");
        self.is_primary = true;
        self.connected = false;
        self.leader_id = self.id;
        loop {
            let msg_rx = cmd_rx.recv().fuse();
            pin!(msg_rx);

            let accept_connection = listener.accept().fuse();
            pin!(accept_connection);

            select! {
                // From Local
                cmd = msg_rx => {
                    match cmd {
                        Some(cmd) => {
                            self.handle_command(cmd).await?;
                        }
                        None => {
                            log::error!("None command received");
                            return Err(io::Error::new(io::ErrorKind::Other, "None Command Received"));
                        }
                    }
                }
                // Accept Connection
                accepted = accept_connection => {
                    match accepted {
                        Ok((stream, _)) => {
                            self.handle_accepted_stream(&stream, true).await?;
                            return Ok(stream);
                        }
                        Err(err) => {
                            log::error!("Accept error {}", err);
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    /// Processing of the replica stream logic after connection has been established
    pub async fn replica_stream_process(
        &mut self,
        cmd_rx: &mut UnboundedReceiver<Command>,
        predecessor_stream: &TcpStream,
        listener: &TcpListener,
    ) -> io::Result<Option<TcpStream>> {
        loop {
            let msg_rx = cmd_rx.recv().fuse();
            pin!(msg_rx);

            let stream_ready = predecessor_stream.readable().fuse();
            pin!(stream_ready);

            let accept_connection = listener.accept().fuse();
            pin!(accept_connection);

            select! {
                // From Local
                cmd = msg_rx => {
                    match cmd {
                        Some(cmd) => {
                            self.handle_command(cmd).await?;
                        }
                        None => {
                            log::error!("None command received");
                            return Err(io::Error::new(io::ErrorKind::Other, "None Command Received"));
                        }
                    }
                }
                // From Sockets
                response = stream_ready => {
                    match response {
                        Ok(_) => {
                            self.handle_socket(&predecessor_stream).await?;
                        }
                        Err(err) => {
                            log::error!("Socket error {}", err);
                        }
                    }
                }
                // Accept Connection
                accepted = accept_connection => {
                    match accepted {
                        Ok((stream, _)) => {
                            self.handle_accepted_stream(&stream, false).await?;
                            return Ok(Some(stream));
                        }
                        Err(err) => {
                            log::error!("Accept error {}", err);
                        }
                    }
                }
            }
        }
    }

    pub async fn run(mut self, mut cmd_rx: UnboundedReceiver<Command>) -> io::Result<()> {
        let backend = &self.connections_info.backend;
        let addr = ConnectionInfoDict::get_socket_addr(backend, self.id);
        let listener = TcpListener::bind(addr).await?;

        let mut predecessor_stream = match self.try_connect().await {
            true => {
                let my_replica_str =
                    ConnectionInfoDict::get_own_info_str(&self.connections_info.backend, self.id);
                self.send_successor(my_replica_str.as_bytes()).await?;
                let (stream, _) = listener.accept().await?;
                stream
            }
            false => {
                self.event_loop_until_connect(&mut cmd_rx, &listener)
                    .await?
            }
        };

        loop {
            match self
                .replica_stream_process(&mut cmd_rx, &predecessor_stream, &listener)
                .await
            {
                Ok(maybe_stream) => match maybe_stream {
                    Some(stream) => {
                        log::info!("Changing predecessor stream");
                        predecessor_stream = stream;
                    }
                    None => {
                        log::info!("Rerunning");
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WriteZero => {
                    match self
                        .handle_predecessor_disconnect(&mut cmd_rx, &listener)
                        .await
                    {
                        Ok(stream) => {
                            predecessor_stream = stream;
                        }
                        Err(e) => {
                            log::error!("Disconnect could not be handled with error {}", e);
                            break;
                        }
                    }
                }
                Err(ref e) => {
                    log::info!("Received error {} attempting to exit", e);
                    break;
                }
            }
        }

        return Ok(());
    }
}

/// Handle and command sender for manager.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ReplicaHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ReplicaHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<String>) -> usize {
        log::info!("Replica Handle connect");
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: manager should not have been dropped
        self.cmd_tx
            .send(Command::Connect { conn_tx, res_tx })
            .unwrap();

        // unwrap: manager does not drop out response channel
        res_rx.await.unwrap()
    }

    /// Send message to manager
    pub async fn send_message(&self, msg: impl Into<String>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                res_tx,
            })
            .unwrap();

        res_rx.await.unwrap();
    }

    /// Unregister message sender
    pub fn disconnect(&self, conn: usize) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}
