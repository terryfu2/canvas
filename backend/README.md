## Setup (Windows)

### Install rustup
https://www.rust-lang.org/tools/install

### Install `rust-analyzer` (if you use vscode)
https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer

### Install postgres
https://www.enterprisedb.com/downloads/postgres-postgresql-downloads

## Create multiple local instances of postgres
```
Make sure the postgres cli tools are installed and you have them added to PATH.
In new terminal
initdb -D c:\Data\PostgresInstance2 -W -A md5
Uncomment and change port in c:\Data\PostgresInstance2\postgresql.conf to 5433
pg_ctl start -D c:\Data\PostgresInstance2
pg_ctl register -N postgres2 -D c:\Data\PostgresInstance2
psql  -d template1 --port=5433
CREATE DATABASE postgres;
CREATE USER postgres WITH ENCRYPTED PASSWORD '<your_pass>';
GRANT ALL PRIVILEGES ON DATABASE postgres TO postgres;
GRANT ALL ON SCHEMA public TO postgres;

For 3 (Doesn't work rn)
initdb -D c:\Data\PostgresInstance3 -W -A md5
Uncomment and change port in c:\Data\PostgresInstance2\postgresql.conf to 5434
pg_ctl start -D c:\Data\PostgresInstance3
pg_ctl register -N postgres3 -D c:\Data\PostgresInstance3
psql  -d template1 --port=5434
CREATE DATABASE postgres;
CREATE USER postgres WITH ENCRYPTED PASSWORD '<your_pass>';
GRANT ALL PRIVILEGES ON DATABASE postgres TO postgres;
GRANT ALL ON SCHEMA public TO postgres;

These database shut down on every Crtl C and needs to be restarted with
pg_ctl start -D c:\Data\PostgresInstance<X>
https://postgrespro.com/list/thread-id/1835410
That might fix it, but im too lazy to try
```

# Start application
```bash
cargo run
(on another bash)
cargo run --config ./.cargo/config1.toml
```

