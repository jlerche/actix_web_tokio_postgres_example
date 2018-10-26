use std::io;

use actix;
use actix::{WrapFuture, Actor, fut, ActorFuture, ContextFutureSpawner, AsyncContext};
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
                .and_then(|(mut cl, conn), act, ctx| {
                    ctx.wait(
                        cl.prepare("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY NOT NULL, email VARCHAR(100) NOT NULL);")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(move |st, actt, ctxx| {
                                ctxx.wait(
                                    cl.execute(&st, &[]) // .poll().unwrap() also works, i don't like it though for some reason
                                        .map_err(|_| ())
                                        .into_actor(actt)
                                        .and_then(|_,_,_| {fut::ok(())})
                                );
                                fut::ok(())
                            }),
                    );
                    actix::Arbiter::spawn(conn.map_err(|e| panic!("{}", e)));
                    fut::ok(())
                }).wait(ctx);
            act
        })
    }
}