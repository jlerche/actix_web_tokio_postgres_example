use actix_web::{HttpRequest, HttpResponse, FutureResponse, AsyncResponder, HttpMessage, http};
use futures::Future;

use apps::app::AppState;
use database::{db, models};

#[derive(Deserialize, Serialize, Debug)]
pub struct NewUser {
    email: String,
}

pub fn user_create_handler(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let db_chan = req.state().db.clone();
    req.json()
        .from_err()
        .and_then(move |val: NewUser| {
            db_chan.send(db::CreateUser {
                email: val.email
            })
                .from_err()
                .and_then(|res| match res {
                    Ok(user) => Ok(HttpResponse::Ok().json(user)),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        }).responder()
}

pub fn user_detail_handler(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let id: i32 = req.match_info().query("id").unwrap();
    let db_chan = req.state().db.clone();
    db_chan.send(db::GetUser {
        id
    })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::new(http::StatusCode::NOT_FOUND)),
        }).responder()
}

pub fn user_update_handler(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let id: i32 = req.match_info().query("id").unwrap();
    let db_chan = req.state().db.clone();
    req.json()
        .from_err()
        .and_then(move |val: NewUser| {
            db_chan
                .send(db::UpdateUser {
                    user: models::User {
                        id,
                        email: val.email,
                    }
                })
                .from_err()
                .and_then(|res| match res {
                    Ok(user) => Ok(HttpResponse::Ok().json(user)),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        }).responder()
}

pub fn user_delete_handler(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let id: i32 = req.match_info().query("id").unwrap();
    let db_chan = req.state().db.clone();
    db_chan.send(db::DeleteUser {
        id
    })
        .from_err()
        .and_then(|res| match res {
            Ok(_) => Ok(HttpResponse::build(http::StatusCode::OK).body("User deleted")),
            Err(_) => Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)),
        }).responder()
}

pub fn user_list_handler(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let db_chan = req.state().db.clone();
    db_chan.send(db::ListUsers)
        .from_err()
        .and_then(|res| match res {
            Ok(users) => Ok(HttpResponse::Ok().json(users)),
            Err(_) => Ok(HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)),
        }).responder()
}
