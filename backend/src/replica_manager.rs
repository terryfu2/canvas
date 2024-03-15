//! A multi-room chat server.
use tokio::net::{TcpStream, TcpListener};
use std::io;
use deadpool_postgres::Pool;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot};
use std::net::{Ipv4Addr, SocketAddrV4};
use crate::Msg;
use futures_util::future::{select, Either};
use tokio::pin;
use std::str;
use serde_json;
use crate::pixel::Pixel;
use std::fs::File;
use std::io::BufReader;
use rand::{thread_rng, Rng as _};
use std::collections::HashMap;

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfo {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReplicaInfo {
    pub id: u16,
    pub public_address: String,
    pub public_port: u16,
    pub address: String,
    pub socket_port: u16,
    pub active: bool
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoDict {
    pub frontend: ConnectionInfo,
    pub proxy: ConnectionInfo,
    pub backend: Vec<ReplicaInfo>
}

impl ConnectionInfoDict {
    fn get_active_replicas(backend: &Vec<ReplicaInfo>) -> usize {
        let mut active_replicas = 0;
        for i in backend.iter() {
            if i.active {
                active_replicas += 1;
            }
        }
        return active_replicas;
    }

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

    fn set_active(backend: &mut Vec<ReplicaInfo>, id: u16, active: bool) {
        let index = backend.iter().position(|r| r.id == id).unwrap();
        backend[index as usize].active = active;
    }
}

// TODO calc max size or find it experimentally
const REPLICA_BUFFER_SIZE: usize = 102400;

fn connections_file() -> String {
    std::env::var("CONNECTIONS_FILE").unwrap_or_else(|_| "../../process_connections.json".into())
}

fn proc_id() -> u16 {
    std::env::var("ID").unwrap_or_else(|_| "0".into()).parse::<u16>().unwrap_or_else(|_| 0)
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

    predecessor_listener: Option<TcpListener>,
    leader_id: u16
}

impl ReplicaManager {
    pub fn new(is_primary: bool, db: Pool, cmd_tx: mpsc::UnboundedSender<Command>) -> (Self, ReplicaHandle) {
        let id = proc_id();
        log::info!("Proc id is {}", id);
        // Open the file in read-only mode with buffer.
        let file = File::open(connections_file()).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file
        let connections_info: ConnectionInfoDict = serde_json::from_reader(reader).unwrap();

        
        let successor_id= ConnectionInfoDict::get_successor_id(&connections_info.backend, id);
        let predecessor_id = ConnectionInfoDict::get_predecessor_id(&connections_info.backend, id);

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
                // predecessor_stream: None
                election_running: false,
                connections_info,
                predecessor_id: predecessor_id,
                successor_id: successor_id,
                predecessor_listener: None,
                leader_id
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
                    Some(pixels) => {
                        self.handle_all_pixels_msg(pixels.to_string()).await
                    }
                    None => {
                        log::info!("No pixels provided to all pixels update");
                    }
                }
                // '/election election/leader id'
                "/election" => match cmd_args.next() {
                    Some(id) => {
                        self.handle_election_msg(id.to_string()).await?
                    }
                    None => {
                        log::info!("No ID provided for election");
                    }
                }

                "/disconnect" => match cmd_args.next() {
                    Some(id) => {
                        self.handle_disconnect_msg(id.to_string()).await?
                    }
                    None => {
                        log::info!("No ID provided for disconnect");
                    }
                }
                
    
                _ => {
                    log::info!("Unknown command {}", msg);
                }
            }
        } else {
            self.handle_pixel_msg(msg).await;
        }
        return Ok(())
    }

    /// Normal pixel update, add it to db
    /// Really these should return errors too, but to lazy to box
    pub async fn handle_pixel_msg(&mut self, msg: String) {
        if self.is_primary {
            // We already updated our database, do nothing
            return;
        }
        log::info!("Pixel update received {}", msg);
        let db = self.db.get().await.unwrap();
        Pixel::insert(db, msg.clone()).await.unwrap();

        if !self.is_primary {
            log::info!("Replica sent to successor {}", msg);
            self.send_successor(msg.as_bytes()).await.unwrap();
        } else {
            log::info!("Ignored msg {}", msg);
        }
    }

    /// Clear and set the entire database to list of pixels provided
    /// Really these should return errors too, but to lazy to box
    pub async fn handle_all_pixels_msg(&mut self, msg: String) {
        if self.is_primary {
            // We already updated our database, do nothing
            return;
        }
        log::info!("All pixels update received {}", msg);
        let db = self.db.get().await.unwrap();
        Pixel::update_all(db, &msg).await.unwrap();

        if !self.is_primary {
            log::info!("Replica sent to successor {}", msg);
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
        log::info!("Election msg received {}", msg);
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
                    log::error!("Election error {}", e);
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
            log::info!("New leader elected {}", id);
            self.leader_id = id;
            if id != self.id {
                self.is_primary = false;
                let election_message = format!("/election leader {}", id);
                log::info!("Election sending {}...", election_message);
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
                log::info!("Election sending {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
            if id < self.id && !self.election_running {
                self.election_running = true;
                let election_message = format!("/election election {}", self.id);
                log::info!("Election sending {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
            if id == self.id {
                let election_message = format!("/election leader {}", self.id);
                log::info!("Election sending {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            }
        } else {
            log::error!("Election error unrecognized election type: {}", election_type);
        }
        return Ok(())

    }

    /// Received a disconnect message. If its our successor get a new connection
    pub async fn handle_disconnect_msg(&mut self, msg: String) -> io::Result<()> {
        log::info!("Disconnect msg received {}", msg);

        // Get id that disconnected
        let id = match msg.parse::<u16>() {
            Ok(val) => val,
            Err(e) => {
                log::error!("Disconnect parse error {} using value {}", e, self.successor_id);
                self.successor_id
            }
        };
        
        ConnectionInfoDict::set_active(&mut self.connections_info.backend, id, false);


        if id == self.successor_id {
            let new_id = ConnectionInfoDict::get_successor_id(&self.connections_info.backend, self.id);
            log::info!("Found new successor attempting to establish connection. id: {}", new_id);
            self.successor_id = new_id;
            log::info!("New successor id {}", self.predecessor_id);
            self.successor_stream = Some(TcpStream::connect(ConnectionInfoDict::get_socket_addr(&self.connections_info.backend, new_id)).await?);
            log::info!("Connected");

            // Either this one or the other one needs to start an election
            // I am not sure whats better
            if id == self.leader_id {
                self.initiate_election().await?;
            }
            return Ok(())
        } else { // Forward
            let disconnect_msg = format!("/disconnect {}", id);
            self.send_successor(disconnect_msg.as_bytes()).await
        }
    }

    /// Connect the successor and predecessor.
    /// Probably shouldn't return predecessor, but it does until
    /// I can figure out the borrowing more
    pub async fn connect_streams(&mut self) -> io::Result<TcpStream> {
        let id = self.id;
        let successor_stream: TcpStream;
        let predecessor_stream: TcpStream;
        let backend = &self.connections_info.backend;
        let predecessor_addr = ConnectionInfoDict::get_socket_addr(backend, self.predecessor_id);
        let successor_addr = ConnectionInfoDict::get_socket_addr(backend, self.successor_id);
        let my_addr = ConnectionInfoDict::get_socket_addr(backend, id);

        // We need to change this to a select async. That would be cool
        // It would have to poll connect whilst listening. Probably too complex for this project honestly
        if id == 2 {
            let listener = TcpListener::bind(my_addr).await?;
            log::info!("Listening on  {}", my_addr.port());

            log::info!("Waiting for successor port: {}", successor_addr.port());
            (successor_stream, _) = listener.accept().await?;

            log::info!("Waiting for predecessor port: {}", predecessor_addr.port());
            (predecessor_stream, _) = listener.accept().await?;

            self.predecessor_listener = Some(listener);
        } else if id == 1 {
            let listener = TcpListener::bind(my_addr).await?;
            log::info!("Listening on  {}", my_addr.port());

            log::info!("if: Connecting to successor port: {}...", predecessor_addr.port());
            predecessor_stream = TcpStream::connect(predecessor_addr).await?;

            log::info!("if: Waiting for predecessor port: {}", successor_addr.port());
            (successor_stream, _) = listener.accept().await?;

            self.predecessor_listener = Some(listener);

        } else { // Last replica
            let listener = TcpListener::bind(my_addr).await?;
            log::info!("Listening on  {}", my_addr.port());

            log::info!("if: Connecting to successor port: {}...", successor_addr.port());
            successor_stream = TcpStream::connect(successor_addr).await?;

            log::info!("if: Connecting to predecessor port: {}...", predecessor_addr.port());
            predecessor_stream = TcpStream::connect(predecessor_addr).await?;

            self.predecessor_listener = Some(listener);
        }
        self.successor_stream = Some(successor_stream);

        log::info!("Connected!");
        return Ok(predecessor_stream)
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
    pub async fn handle_predecessor_disconnect(&mut self) -> io::Result<TcpStream> {
        log::info!("Attempting to connect to new predecessor");
        ConnectionInfoDict::set_active(&mut self.connections_info.backend, self.predecessor_id, false);
        if ConnectionInfoDict::get_active_replicas(&self.connections_info.backend) == 1 {
            log::error!("We cannot recover replica manager proc");
            return Err(io::ErrorKind::Other.into())
        }

        let msg = format!("/disconnect {}", self.predecessor_id);
        self.send_successor(msg.as_bytes()).await?;
        self.predecessor_id = ConnectionInfoDict::get_predecessor_id(&mut self.connections_info.backend, self.id);
        log::info!("New predecessor id {}", self.predecessor_id);

        match self.predecessor_listener.as_mut() {
            Some(listener) => {
                log::info!("Listening for new connection");
                let (predecessor_stream, _) = listener.accept().await?;
                log::info!("Connected!");
                return Ok(predecessor_stream);
            }
            None => {
                let my_addr = ConnectionInfoDict::get_socket_addr(&self.connections_info.backend, self.id);
                let listener = TcpListener::bind(my_addr).await?;
                let (predecessor_stream, _) = listener.accept().await?;
                self.predecessor_listener = Some(listener);
                return Ok(predecessor_stream);
            }
        }
    }

    /// Let all ws sessions know that we are the new primary so they can forward that to their proxies
    async fn send_primary_to_ws(&self) {
        let msg = format!("primary");

        for (id, session) in &self.sessions {
            log::info!("Sending primary to session {}", id);
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

    /// Processing of the replica stream logic after connection has been established
    pub async fn replica_stream_process(&mut self, cmd_rx: &mut UnboundedReceiver<Command>,predecessor_stream: &TcpStream) -> io::Result<()> {
        loop {
            let msg_rx = cmd_rx.recv();
            pin!(msg_rx);

            let mut predecessor_buf = vec![0; REPLICA_BUFFER_SIZE];
            let stream_ready = predecessor_stream.readable();
            pin!(stream_ready);

            match select(msg_rx, stream_ready).await {
                // From Local
                Either::Left((Some(cmd), _)) => {
                    match cmd {
                        Command::Connect { conn_tx, res_tx } => {
                            let conn_id = self.register_session(conn_tx).await;
                            let _ = res_tx.send(conn_id);
                        }
                        Command::Message { msg, res_tx } => {
                            log::info!("Msg received {}", msg);
                            self.send_successor(msg.as_bytes()).await?;
                            let _ = res_tx.send(());
                        }
                        Command::Disconnect { conn } => {
                            self.unregister_session(conn).await;
                        }
                    }
                },
                // From Sockets
                Either::Right((response, _)) => {
                    match response {
                        Ok(_) => {
                            match predecessor_stream.try_read(&mut predecessor_buf) {
                                Ok(n) => {
                                    // If the predecessor_stream's proc crashes we get some issues
                                    if n == 0 {
                                        log::error!("Err received size 0 likely predecessor crashed");
                                        return Err(io::ErrorKind::WriteZero.into())
                                    }

                                    predecessor_buf.truncate(n);
                                    let predecessor_msg = match str::from_utf8(&predecessor_buf) {
                                        Ok(v) => v,
                                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                                    };
                                    log::info!("Received msg from socket {}", predecessor_msg);
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
                        }
                        Err(err) => {
                            log::error!("Socket error {}", err);
                            break;
                        }
                    }
                },

                Either::Left((None, _)) => {
                    break;
                }
            }
        }

        return Ok(())
    }

    /// Send a message to the predecessor
    // pub async fn send_predecessor(&mut self, msg: &[u8]) -> io::Result<()> {
    //     match self.predecessor_stream.as_mut() {
    //         Some(predecessor_stream) => predecessor_stream.write_all(msg).await,
    //         None => {
    //             log::error!("Attempted to predecessor write with no connection");
    //             // Maybe could recover and not panic here
    //             panic!("Attempted to predecessor write with no connection");
    //         }
    //     }
    // }

    pub async fn run(mut self, mut cmd_rx: UnboundedReceiver<Command>) -> io::Result<()> {

        // We should get a connect from the ws thread
        // log::error!("Waiting for connect from ws...");
        // match cmd_rx.recv().await {
        //     Some(cmd) => {
        //         match cmd {
        //             Command::Connect { conn_tx, res_tx } => {
        //                 let conn_id = self.register_session(conn_tx).await;
        //                 let _ = res_tx.send(conn_id);
        //             }
        //             Command::Message { msg, res_tx } => {
        //                 log::error!("Got message instead of connect {}", msg);
        //                 let _ = res_tx.send(());
        //             }
        //         }
        //     }
        //     None  => {
        //         log::error!("No connect from ws thread");
        //     }
        // }
        // If we only have one connection no point in running this
        if ConnectionInfoDict::get_active_replicas(&self.connections_info.backend) == 1 {
            // Exit gracefully
            return Ok(());
        }

        let mut predecessor_stream: TcpStream = self.connect_streams().await?;

        let db = self.db.get().await.unwrap();

        // Do an initial sync of all replicas
        if self.is_primary {
            let pixels = Pixel::all(&**db).await.unwrap();
            let mut pixels_str = serde_json::to_string(&pixels).unwrap();
            pixels_str = format!("/all_pixels {}", pixels_str);
            let pixels_bytes = pixels_str.as_bytes();
            log::info!("Sending {} bytes", pixels_bytes.len());
            self.send_successor(pixels_bytes).await?;
        } else {
            let mut buf = vec![0; REPLICA_BUFFER_SIZE];
            predecessor_stream.readable().await?;
            match predecessor_stream.try_read(&mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    let msg = match str::from_utf8(&buf) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    self.handle_replica_msg(msg.to_string()).await?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // This gets called after every update idk why
                    log::error!("Err would block (Don't worry about this I think) {}", e);
                }
                Err(e) => {
                    log::error!("Err try_read {}", e);
                    return Err(e.into());
                }
            };
        }

        loop {
            match self.replica_stream_process(&mut cmd_rx, &predecessor_stream).await {
                Ok(_) => {
                    log::info!("Exited gracefully");
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WriteZero => {
                    match self.handle_predecessor_disconnect().await {
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

        return Ok(())
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

    /// Send message to next replica
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
