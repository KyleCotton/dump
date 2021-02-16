use crate::auth::Token;
use crate::user::User;

use actix_web::{post, web, web::Data, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    pub user_name: String,
    pub password: String,
}

#[post("/auth")]
pub async fn auth(conn: Data<PgPool>, user: web::Json<AuthRequest>) -> HttpResponse {
    let user = match User::login(&conn, &user.user_name, &user.password).await {
        Ok(u) => u,
        Err(e) => return e.into(),
    };

    Token::new(&user)
        .await
        .map_or_else(|e| e.into(), |t| HttpResponse::Ok().json(t))
}
