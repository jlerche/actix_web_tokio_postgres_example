extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate env_logger;
extern crate tokio_postgres;
#[macro_use]
extern crate serde_derive;

mod database;
mod apps;

use actix_web::{HttpRequest, HttpResponse, FutureResponse, http, AsyncResponder, middleware};

use apps::app::AppState;
use database::db::PgConnection;
use futures::future;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let sys = actix::System::new("local_dev");
    let db_url = "postgres://dev:dev@localhost/actix_web_tokio_postgres_example_development";

    actix_web::server::new(move || {
        let addr = PgConnection::connect(db_url);

        actix_web::App::with_state(AppState{db: addr})
            .middleware(middleware::Logger::default())
            .resource("/healthz", |r| r.method(http::Method::GET).a(|_: &HttpRequest<AppState>| -> FutureResponse<HttpResponse> {
                let fut = future::ok(HttpResponse::new(http::StatusCode::OK));
                fut.responder()
            }))
    })
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
