use crate::command::{Command, Instruction};
use crate::user::User;

use actix_web::{get, post, web, web::Data, HttpResponse};
use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRequest {
    // robot_serial_number: String,
    #[serde(with = "ts_seconds")]
    time_issued: chrono::DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    time_instruction: chrono::DateTime<Utc>,
    instruction: Instruction,
    // TODO: Include the frequency of a command
    // frequency: i64,
}

#[post("/command")]
pub async fn create_command(
    conn: Data<PgPool>,
    user: User,
    cmd: web::Json<CommandRequest>,
) -> HttpResponse {
    Command::new(
        &conn,
        &user.robot_serial_number,
        cmd.time_issued,
        cmd.time_instruction,
        &cmd.instruction,
    )
    .await
    .map_or_else(|e| e.into(), |cmd| HttpResponse::Ok().json(cmd))
}
