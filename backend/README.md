Rust websocket backend with a postgres server.
Heavily inspired from actix actprless weboscket websocket https://github.com/actix/examples/blob/1e1767135d87bba84747c397fa8aa4d9904460c9/websockets/echo-actorless/src/main.rs
and docker example https://github.com/docker/awesome-compose/tree/master/react-rust-postgres
```
To run locally you need to install postgres and start a database.
In config.toml, change the PG_PASSWORD to the password you setup when downloading postgres.
You then must download rust. 
Then simply run `cargo run` to run the server
```
## Setup (After dependencies are installed)
```
cd backend
cargo run
```