use actix_web::{get, web, App, HttpResponse, HttpServer};
use deadpool_postgres::Pool;

mod postgres;
mod pixel;

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
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch pixels: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch pixels");
        }
    }
}

// #[post("/pixel")]
// async fn get_pixels(pool: web::Data<Pool>) -> HttpResponse {
//     let client = match pool.get().await {
//         Ok(client) => client,
//         Err(err) => {
//             log::debug!("unable to get postgres client: {:?}", err);
//             return HttpResponse::InternalServerError().json("unable to get postgres client");
//         }
//     };
//     match pixel::Pixel::all(&**client).await {
//         Ok(list) => HttpResponse::Ok().json(list),
//         Err(err) => {
//             log::debug!("unable to fetch pixels: {:?}", err);
//             return HttpResponse::InternalServerError().json("unable to fetch pixels");
//         }
//     }
// }

fn address() -> String {
    std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let address = address();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            .service(get_pixels)
    })
    .bind(&address)?
    .run()
    .await
}
