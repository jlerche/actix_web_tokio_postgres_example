use std::io;

use actix;
use actix::{WrapFuture, Actor, fut, ActorFuture, ContextFutureSpawner};
use futures::{Future};
use tokio_postgres;

pub struct PgConnection {
    client: Option<tokio_postgres::Client>
}

impl actix::Actor for PgConnection {
    type Context = actix::Context<Self>;
}

impl PgConnection {
    pub fn connect(db_url: &str) -> actix::Addr<PgConnection> {
        let hs = tokio_postgres::connect(db_url.parse().unwrap(), tokio_postgres::TlsMode::None);
        PgConnection::create(move |ctx| {
            let act = PgConnection {
                client: None,
            };

            hs.map_err(|_| panic!("cannot connect to postgresql"))
                .into_actor(&act)
                .and_then(|(cl, conn), act, ctx| {
                    actix::Arbiter::spawn(conn.map_err(|e| panic!("{}", e)));
                    fut::ok(())
                }).wait(ctx);
            act
        })
    }
}