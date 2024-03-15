use actix_web::{
    web::Json, get, post, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, rt,
};
use deadpool_postgres::Pool;
use actix_cors::Cors;
use tokio::{
    task::spawn,
    try_join,
    sync::mpsc
};
mod postgres;
mod pixel;
mod handler;
use serde_json::json;

mod replica_manager;

pub use self::replica_manager::{ReplicaHandle, ReplicaManager};

/// Connection ID.
pub type ConnId = usize;

/// Message sent to a replica.
pub type Msg = String;

#[get("/canvas")]
async fn get_pixels(pool: web::Data<Pool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match pixel::Pixel::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json((json!({
            "command": "get_pixels",
            "payload": list,
        }))),
        Err(err) => {
            log::debug!("unable to fetch pixels: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch pixels");
        }
    }
}

// Entry point for our websocket route
async fn canvas_route(
    req: HttpRequest, stream: web::Payload, pool: web::Data<Pool>, replica_handle: web::Data<ReplicaHandle>) -> Result<HttpResponse, Error> {
        let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

        // spawn websocket handler (and don't await it) so that the response is returned immediately
        rt::spawn(handler::canvas_ws((**replica_handle).clone(), session, msg_stream, pool));

        Ok(res)
    }

#[post("/pixel")]
async fn set_pixel(pool: web::Data<Pool>, data: Json<pixel::Pixel>) -> HttpResponse {
    log::debug!("pixel data: {:?}", data);
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match pixel::Pixel::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch pixels: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch pixels");
        }
    }
}

fn address() -> String {
    std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into())
}

fn is_primary() -> bool {
    std::env::var("PRIMARY").unwrap_or_else(|_| "false".into()) == "true"
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    let is_primary = is_primary();
    let (replica_handler, tx) = ReplicaManager::new(is_primary, pg_pool.clone(), cmd_tx);

    let chat_server = spawn(replica_handler.run(cmd_rx));

    let address = address();
    log::info!("address {}", address);
    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pg_pool.clone()))
            .app_data(web::Data::new(tx.clone()))
            .service(get_pixels)
            .service(set_pixel)
            // websocket route
            .service(web::resource("/ws").route(web::get().to(canvas_route)))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(&address)?
    .run();

    try_join!(http_server, async move { chat_server.await.unwrap() })?;

    Ok(())
}
