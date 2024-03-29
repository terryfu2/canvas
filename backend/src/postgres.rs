use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tokio_postgres_migration::Migration;

const SCRIPTS_UP: [(&str, &str); 1] = [(
    "0001_create-database",
    include_str!("../migrations/0001_create-database.sql"),
)];

fn create_config() -> Config {
    let mut cfg = Config::new();
    if let Ok(host) = std::env::var("PG_HOST") {
        cfg.host = Some(host);
    }
    if let Ok(dbname) = std::env::var("PG_DBNAME") {
        cfg.dbname = Some(dbname);
    }
    if let Ok(user) = std::env::var("PG_USER") {
        cfg.user = Some(user);
    }
    if let Ok(password) = std::env::var("PG_PASSWORD") {
        cfg.password = Some(password);
    }
    if let Ok(port) = std::env::var("PG_PORT") {
        cfg.port = Some(port.parse::<u16>().unwrap());
    }
    cfg
}

pub fn create_pool() -> Pool {
    create_config()
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("couldn't create postgres pool")
}

pub async fn migrate_up(pool: &Pool) {
    let mut client = pool.get().await.expect("couldn't get postgres client");
    let migration = Migration::new("migrations".to_string());
    migration
        .up(&mut **client, &SCRIPTS_UP)
        .await
        .expect("couldn't run migrations");
}
