use crate::user::User;

use actix_web::{post, web, web::Data, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRequest {
    pub user_name: String,
    pub password: String,
    pub robot_serial_number: String,
}

#[post("/user")]
pub async fn create_user(conn: Data<PgPool>, user: web::Json<UserRequest>) -> HttpResponse {
    User::new(
        &conn,
        &user.user_name,
        &user.password,
        &user.robot_serial_number,
    )
    .await
    .map_or_else(|e| e.into(), |user| HttpResponse::Ok().json(user))
}
