use crate::error::ApiError;
use crate::robot::Robot;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    user_id: i64,
    pub user_name: String,
    password_hash: String,
    pub robot_serial_number: String,
}

impl User {
    pub async fn new(
        conn: &PgPool,
        user_name: &str,
        password: &str,
        robot_serial_number: &str,
    ) -> Result<Self, ApiError> {
        // TODO: fix this
        let user_name = user_name.to_string();
        let robot_serial_number = robot_serial_number.to_string();

        let password_hash = hash(password, DEFAULT_COST).map_err(|_| ApiError::HashingFailed)?;

        Robot::new(conn, &robot_serial_number).await?;

        let user_id = sqlx::query!(
            r#"
INSERT INTO users (user_name, password_hash, robot_serial_number)
VALUES ( $1, $2, $3 )
RETURNING user_id
        "#,
            user_name,
            password_hash,
            robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            println!("User Insert: {:?}", e);
            ApiError::DatabaseConnFailed
        })?
        .user_id;

        Ok(Self {
            user_id,
            user_name,
            password_hash,
            robot_serial_number,
        })
    }

    pub async fn search_by_username(
        conn: &PgPool,
        user_name: &str,
    ) -> Result<Option<Self>, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM users U
WHERE U.user_name = $1
"#,
            user_name
        )
        .fetch_optional(conn)
        .await
        .map(|user| {
            user.map(|u| Self {
                user_id: u.user_id,
                user_name: u.user_name,
                password_hash: u.password_hash,
                robot_serial_number: u.robot_serial_number,
            })
        })
        .map_err(|_| ApiError::DatabaseConnFailed)
    }

    pub async fn login(conn: &PgPool, user_name: &str, password: &str) -> Result<Self, ApiError> {
        let user = Self::search_by_username(&conn, user_name)
            .await?
            .ok_or(ApiError::LoginFailedUserNotExist)?;

        match verify(&password, &user.password_hash) {
            Ok(verified) if verified => Ok(user),
            _ => Err(ApiError::LoginFailedPasswordIncorrect),
        }
    }
}
