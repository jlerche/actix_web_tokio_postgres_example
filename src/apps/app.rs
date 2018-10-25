use actix;

use database::db::PgConnection;

pub struct AppState {
    pub db: actix::Addr<PgConnection>,
}