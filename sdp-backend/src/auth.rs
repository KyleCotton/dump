use crate::error::ApiError;
use crate::user::User;
use actix_web::{dev, http::header, web::Data, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use futures_util::future::{err, ok, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tokio::runtime::Runtime;

const JWT_SECRET: &str = "dvafsdvbjdfasv43543578634785jkbv";
const ACTIVE_DAYS: i64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub token: String,
}

// TODO: Only store the username
#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    user_name: String,
    exp: i64,
}

impl Token {
    pub async fn new(user: &User) -> Result<Self, ApiError> {
        let headers = Header::default();
        let encoding_key = EncodingKey::from_secret(JWT_SECRET.as_bytes());
        let expiration = (Utc::now() + Duration::days(ACTIVE_DAYS)).timestamp(); // Expires in 1 day
        let claims = Claims {
            user_name: user.user_name.clone(),
            exp: expiration,
        };

        encode(&headers, &claims, &encoding_key)
            .map(|token| Token { token })
            .map_err(|_| ApiError::DatabaseConnFailed)
    }

    pub async fn validate(&self, conn: &PgPool) -> Result<User, ApiError> {
        // pub fn validate(&self, conn: &PgPool) -> Result<User, ApiError> {
        let user_name = decode::<Claims>(
            &self.token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map(|t| t.claims.user_name)
        .map_err(|_| ApiError::AuthenticationFailed)?;

        match User::search_by_username(conn, &user_name).await {
            Ok(Some(u)) => Ok(u),
            _ => Err(ApiError::AuthenticationFailed),
        }
    }
}

// Using the FromRequest trait to authenticate the user
impl FromRequest for User {
    type Error = ApiError;
    type Future = Ready<Result<Self, ApiError>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let conn = match Data::<PgPool>::from_request(req, payload).into_inner() {
            Ok(d) => d,
            _ => return err(ApiError::DatabaseConnFailed),
        };

        // Get the headers from the request
        // The AUTH header contains the token
        let token = match req
            .headers()
            .get(header::AUTHORIZATION)
            .map(|hv| hv.to_str())
        {
            Some(Ok(t)) => Token {
                token: t.to_string(),
            },
            _ => return err(ApiError::AuthenticationFailed),
        };

        let rt = match Runtime::new() {
            Ok(r) => r,
            _ => return err(ApiError::AuthenticationFailed),
        };

        rt.block_on(async move {
            token
                .validate(&conn)
                .await
                .map_or_else(|e| err(e), |u| ok(u))
        })
    }
}
