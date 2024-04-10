use crate::Msg;
use crate::ReplicaHandle;
use actix_web::web;
use actix_ws::Message;
use deadpool_postgres::Pool;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use std::time::{Duration, Instant};
use tokio::{pin, sync::mpsc, time::interval};

/// How often heartbeat pings are sent.
///
/// Should be half (or less) of the acceptable client timeout.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
pub async fn canvas_ws(
    replica_handle: ReplicaHandle,
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    pool: web::Data<Pool>,
) {
    log::info!("WS connected");

    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel::<Msg>();

    // unwrap: manager is not dropped before the HTTP server
    let conn_id = replica_handle.connect(conn_tx).await;

    let close_reason = loop {
        // most of the futures we process need to be stack-pinned to work with select()

        let tick = interval.tick();
        pin!(tick);

        let msg_rx = conn_rx.recv();
        pin!(msg_rx);

        // TODO: nested select is pretty gross for readability on the match
        let messages = select(msg_stream.next(), msg_rx);
        pin!(messages);

        match select(messages, tick).await {
            // commands & messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                match msg {
                    Message::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        // unwrap:
                        session.pong(&bytes).await.unwrap();
                    }

                    Message::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    Message::Text(text) => {
                        log::debug!("msg: {text:?}");
                        replica_handle.send_message(text).await
                    }

                    Message::Binary(_bin) => {
                        log::warn!("unexpected binary message");
                    }

                    Message::Close(reason) => break reason,

                    _ => {
                        break None;
                    }
                }
            }

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // Got a message from the replica manager. Are we the new primary?
            Either::Left((Either::Right((Some(msg), _)), _)) => {
                log::info!("Message received from replica manager {}", msg);
                if msg == "primary" {
                    log::info!("Sending primary message to ws connection");
                    session.text("primary").await.unwrap();
                } else if msg.starts_with("replicated") {
                    let payload = msg.trim_start_matches("replicated: ");
                    log::info!("Sending replicated message to ws connection");
                    session.text(payload).await.unwrap();
                } else if msg.starts_with("unreplicated") {
                    let payload = msg.trim_start_matches("unreplicated: ");
                    log::info!("Sending unreplicated message to ws connection");
                    session.text(payload).await.unwrap();
                } else {
                    log::error!("Unrecognized msg from replica manager {}", msg);
                }
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => {
                log::info!("Replica manager stopped running. Only we are left");
                break None; // Deal with this better
            }

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    replica_handle.disconnect(conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}
