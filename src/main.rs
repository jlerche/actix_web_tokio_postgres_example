extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate env_logger;
extern crate tokio_postgres;
#[macro_use]
extern crate serde_derive;

mod handlers;
mod database;
mod apps;

use actix_web::{HttpRequest, HttpResponse, FutureResponse, http, AsyncResponder, middleware};

use handlers::user_handlers;
use apps::app::AppState;
use database::db::PgConnection;
use futures::future;
use futures::Future;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("local_dev");
    let db_url = "postgres://dev:dev@localhost/actix_web_tokio_postgres_example_development";
    let addr = PgConnection::connect(db_url);
    let addr_clone = addr.clone();

    actix_web::server::new(move || {
        actix_web::App::with_state(AppState { db: addr_clone.clone() })
            .middleware(middleware::Logger::default())
            .resource("/healthz", |r| r.method(http::Method::GET).a(|_: &HttpRequest<AppState>| -> FutureResponse<HttpResponse> {
                let fut = future::ok(HttpResponse::new(http::StatusCode::OK));
                fut.responder()
            })).scope("/user", |scope| {
            scope
                // middleware might go here
                .resource("", |r| {
                    r.route().filter(actix_web::pred::Get()).a(user_handlers::user_list_handler);
                    r.route().filter(actix_web::pred::Post()).a(user_handlers::user_create_handler);
                })
                .resource("/{id}", |r| {
                    r.route().filter(actix_web::pred::Get()).a(user_handlers::user_detail_handler);
                    r.route().filter(actix_web::pred::Put()).a(user_handlers::user_update_handler);
                    r.route().filter(actix_web::pred::Delete()).a(user_handlers::user_delete_handler);
                })
        })
    })
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    actix::Arbiter::spawn(
        addr.send(database::db::InitializeDatabase)
            .map_err(|_| ())
            .map(|res| {
                println!("{}", res.unwrap());
                ()
            })
    );

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
