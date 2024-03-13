//! A multi-room chat server.
use std::io;
use deadpool_postgres::Pool;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot};
use std::net::{Ipv4Addr, SocketAddrV4};
use crate::Msg;
use futures_util::future::{select, Either};
use tokio::pin;
use tokio::net::{TcpStream, TcpListener};
use std::str;
use crate::pixel::Pixel;

/// A command received by the Replica
#[derive(Debug)]
pub enum Command {
    Message {
        msg: Msg,
        res_tx: oneshot::Sender<()>,
    },
}

const REPLICA_BUFFER_SIZE: usize = 102400;

const ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;

fn predecessor_port() -> u16 {
    std::env::var("PREDECESSOR_PORT").unwrap_or_else(|_| "8000".into()).parse::<u16>().unwrap_or_else(|_| 8000)
}
fn successor_port() -> u16 {
    std::env::var("SUCCESSOR_PORT").unwrap_or_else(|_| "8000".into()).parse::<u16>().unwrap_or_else(|_| 8000)
}
fn port() -> u16 {
    std::env::var("SOCKET_PORT").unwrap_or_else(|_| "8000".into()).parse::<u16>().unwrap_or_else(|_| 8000)
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

    is_primary: bool,

    /// Process id
    id: u16,

    /// Postgres db_connection
    db: Pool,
    
    successor_stream: Option<TcpStream>,

    election_running: bool,

    // We can add this back later
    // predecessor_stream: Option<TcpStream>
}

impl ReplicaManager {
    pub fn new(is_primary: bool, db: Pool, cmd_tx: mpsc::UnboundedSender<Command>) -> (Self, ReplicaHandle) {
        let id = proc_id();
        log::info!("Proc id is {}", id);
        (
            Self {
                is_primary,
                id,
                db,
                successor_stream: None,
                // predecessor_stream: None
                election_running: false,
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
                    Some(pixels) => {
                        self.handle_election_msg(pixels.to_string()).await?
                    }
                    None => {
                        log::info!("No pixels provided to all pixels update");
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
            if id != self.id {
                self.is_primary = false;
                let election_message = format!("/election leader {}", id);
                log::info!("Election sending {}...", election_message);
                self.send_successor(election_message.as_bytes()).await?
            } else {
                log::info!("Election we are the primary");
                self.is_primary = true;
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

    /// Connect the successor and predecessor.
    /// Probably shouldn't return predecessor, but it does until
    /// I can figure out the borrowing more
    pub async fn connect_streams(&mut self) -> io::Result<TcpStream> {
        let id = self.id;
        let successor_stream: TcpStream;
        let predecessor_stream: TcpStream;

        // We need to change this to a select async. That would be cool
        // todo change to handle more than 2 replicas
        if id == 1 {
            let listener = TcpListener::bind(SocketAddrV4::new(ADDR, port())).await?;
            log::info!("Listening on  {}", port());

            log::info!("if: Connecting to successor port: {}...", successor_port());
            successor_stream = TcpStream::connect(SocketAddrV4::new(ADDR, successor_port())).await?;

            log::info!("if: Waiting for predecessor port: {}", predecessor_port());
            (predecessor_stream, _) = listener.accept().await?;

        } else if id == 2 { // Last replica
            log::info!("Listening on  {}", port());

            log::info!("if: Connecting to successor port: {}...", successor_port());
            successor_stream = TcpStream::connect(SocketAddrV4::new(ADDR, successor_port())).await?;

            log::info!("if: Connecting to predecessor port: {}...", predecessor_port());
            predecessor_stream = TcpStream::connect(SocketAddrV4::new(ADDR, predecessor_port())).await?;
        
        } else {
            let listener_predecessor = TcpListener::bind(SocketAddrV4::new(ADDR, port())).await?;
            let listener_successor = TcpListener::bind(SocketAddrV4::new(ADDR, successor_port())).await?;
            log::info!("Listening on  {}", port());

            log::info!("Waiting for predecessor port: {}", predecessor_port());
            (predecessor_stream, _) = listener_predecessor.accept().await?;
            // log::info!("Connected to {}", addr.port());

            log::info!("Waiting for successor port: {}", successor_port());
            (successor_stream, _) = listener_successor.accept().await?;
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

        let predecessor_stream = self.connect_streams().await?;

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
            // We should probably keep reading until n != REPLICA_BUFFER_SIZE, but cant be bothered for now
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
            let msg_rx = cmd_rx.recv();
            pin!(msg_rx);

            let mut predecessor_buf = vec![0; REPLICA_BUFFER_SIZE];
            let stream_ready = predecessor_stream.readable();
            pin!(stream_ready);

            match select(msg_rx, stream_ready).await {
                // From Local
                Either::Left((Some(cmd), _)) => {
                    match cmd {
                        Command::Message { msg, res_tx } => {
                            log::info!("Msg received {}", msg);
                            self.send_successor(msg.as_bytes()).await?;
                            let _ = res_tx.send(());
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
                                        log::error!("Err received size 0 likely a panic");
                                        break;
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

        Ok(())
    }
}

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ReplicaHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ReplicaHandle {

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
}
