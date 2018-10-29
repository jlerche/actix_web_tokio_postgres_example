use std::io;

use actix;
use actix::{WrapFuture, Actor, fut, ActorFuture, ContextFutureSpawner, AsyncContext};
use actix_web;
use futures::{Future, IntoFuture, Stream};
use tokio_postgres;

use database::models;

pub struct PgConnection {
    client: Option<tokio_postgres::Client>,
    create_st: Option<tokio_postgres::Statement>,
    read_st: Option<tokio_postgres::Statement>,
    update_st: Option<tokio_postgres::Statement>,
    delete_st: Option<tokio_postgres::Statement>,
    list_st: Option<tokio_postgres::Statement>,
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
                list_st: None,
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
                        cl.prepare("UPDATE users SET email = $2 WHERE id=$1 RETURNING *;")
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
                    ctx.wait(
                        cl.prepare("SELECT * from users;")
                            .map_err(|_| ())
                            .into_actor(act)
                            .and_then(|statement, act, _ctx| {
                                act.list_st = Some(statement);
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

pub struct CreateUser {
    pub email: String,
}

impl actix::Message for CreateUser {
    type Result = Result<models::User, actix_web::Error>;
}

impl actix::Handler<CreateUser> for PgConnection {
    type Result = actix::ResponseFuture<models::User, actix_web::Error>;

    fn handle(&mut self, msg: CreateUser, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .query(self.create_st.as_ref().unwrap(), &[&msg.email])
                .into_future()
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create user"))
                .and_then(|(row, _query)| {
                    let row = row.unwrap();
                    Ok(
                        models::User {
                            id: row.get(0),
                            email: row.get(1),
                        }
                    )
                }),
        )
    }
}

pub struct GetUser {
    pub id: i32,
}

impl actix::Message for GetUser {
    type Result = Result<models::User, actix_web::Error>;
}

impl actix::Handler<GetUser> for PgConnection {
    type Result = actix::ResponseFuture<models::User, actix_web::Error>;

    fn handle(&mut self, msg: GetUser, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .query(self.read_st.as_ref().unwrap(), &[&msg.id])
                .into_future()
                .map_err(|_| actix_web::error::ErrorInternalServerError("failed to get user"))
                .and_then(|(row, _query)| {
                    let row = row.unwrap();
                    Ok(
                        models::User {
                            id: row.get(0),
                            email: row.get(1),
                        }
                    )
                }),
        )
    }
}

pub struct UpdateUser {
    pub user: models::User,
}

impl actix::Message for UpdateUser {
    type Result = Result<models::User, actix_web::Error>;
}

impl actix::Handler<UpdateUser> for PgConnection {
    type Result = actix::ResponseFuture<models::User, actix_web::Error>;

    fn handle(&mut self, msg: UpdateUser, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .query(self.update_st.as_ref().unwrap(), &[&msg.user.id, &msg.user.email])
                .into_future()
                .map_err(|_| actix_web::error::ErrorInternalServerError("failed to get user"))
                .and_then(|(row, _query)| {
                    let row = row.unwrap();
                    Ok(
                        models::User {
                            id: row.get(0),
                            email: row.get(1),
                        }
                    )
                }),
        )
    }
}

pub struct DeleteUser {
    pub id: i32,
}

impl actix::Message for DeleteUser {
    type Result = Result<(), actix_web::Error>;
}

impl actix::Handler<DeleteUser> for PgConnection {
    type Result = actix::ResponseFuture<(), actix_web::Error>;

    fn handle(&mut self, msg: DeleteUser, _ctx: &mut Self::Context) -> Self::Result {
        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .query(self.delete_st.as_ref().unwrap(), &[&msg.id])
                .into_future()
                .map_err(|_| actix_web::error::ErrorInternalServerError("failed to delete user"))
                .and_then(|(_row, _query)| {
                    Ok(())
                }),
        )
    }
}


pub struct ListUsers;

impl actix::Message for ListUsers {
    type Result = Result<Vec<models::User>, actix_web::Error>;
}

impl actix::Handler<ListUsers> for PgConnection {
    type Result = actix::ResponseFuture<Vec<models::User>, actix_web::Error>;

    fn handle(&mut self, _msg: ListUsers, _ctx: &mut Self::Context) -> Self::Result {
        let users = Vec::with_capacity(4);

        Box::new(
            self.client
                .as_mut()
                .unwrap()
                .query(self.list_st.as_ref().unwrap(), &[])
                .fold(users, move |mut users, row| {
                    users.push(
                        models::User {
                            id: row.get(0),
                            email: row.get(1),
                        }
                    );
                    Ok::<_, tokio_postgres::error::Error>(users)
                })
                .map_err(|_| actix_web::error::ErrorInternalServerError("failed to get users"))
                .and_then(|users| {
                    Ok(users)
                }),
        )
    }
}