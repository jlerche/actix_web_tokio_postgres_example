use std::io;

use tokio_postgres::{connect, Client};

pub struct PgConnection {
    client: Option<Client>
}