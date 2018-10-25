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

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

}
