use std::io;

use actix;
use actix::{WrapFuture, Actor, fut, ActorFuture, ContextFutureSpawner, AsyncContext};
use futures::{Future, IntoFuture};
use tokio_postgres;

pub struct PgConnection {
    client: Option<tokio_postgres::Client>,
    create_st: Option<tokio_postgres::Statement>,
    read_st: Option<tokio_postgres::Statement>,
    update_st: Option<tokio_postgres::Statement>,
    delete_st: Option<tokio_postgres::Statement>,
    create_table_st: Option<tokio_postgres::Statement>,
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
                create_st: None,
                read_st: None,
                update_st: None,
                delete_st: None,
                create_table_st: None,
            };

            hs.map_err(|_| panic!("cannot connect to postgresql"))
                .into_actor(&act)
                .and_then(|(mut cl, conn), act, ctx| {
                    ctx.wait(
                        cl.prepare("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY NOT NULL, email VARCHAR(100) NOT NULL);")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|st, act, _| {
//                                ctxx.wait(
//                                    cl.execute(&st, &[]) // .poll().unwrap() also works, i don't like it though for some reason
//                                        .map_err(|_| ())
//                                        .into_actor(actt)
//                                        .and_then(|_,_,_| {fut::ok(())})
//                                );
                                act.create_table_st = Some(st);
                                fut::ok(())
                            }),
                    );
                    ctx.wait(
                        cl.prepare("INSERT INTO users (email) VALUES ($1) RETURNING *;")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|statement, act, _ctxx| {
                                act.create_st = Some(statement);
                                fut::ok(())
                            }),
                    );
                    ctx.wait(
                        cl.prepare("SELECT * FROM users WHERE id=$1;")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|statement, act, _ctx| {
                                act.read_st = Some(statement);
                                fut::ok(())
                            }),
                    );
                    ctx.wait(
                        cl.prepare("UPDATE users SET email = $2 WHERE id=$1;")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|statement, act, _ctx| {
                                act.update_st = Some(statement);
                                fut::ok(())
                            }),
                    );
                    ctx.wait(
                        cl.prepare("DELETE FROM users WHERE id = $1;")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|statement, act, _ctx| {
                                act.delete_st = Some(statement);
                                fut::ok(())
                            }),
                    );
                    act.client = Some(cl);
                    actix::Arbiter::spawn(conn.map_err(|e| panic!("{}", e)));
                    fut::ok(())
                }).wait(ctx);
            act
        })
    }
}

pub struct InitializeDatabase;

impl actix::Message for InitializeDatabase {
    type Result = io::Result<u64>;
}

impl actix::Handler<InitializeDatabase> for PgConnection {
    type Result = actix::ResponseFuture<u64, io::Error>;

    fn handle(&mut self, _msg: InitializeDatabase, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .execute(self.create_table_st.as_ref().unwrap(), &[])
                .into_future()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                .and_then(|num_rows| {
                    Ok(num_rows)
                }),
        )
    }
}