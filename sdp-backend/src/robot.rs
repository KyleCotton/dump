use crate::command::Command;
use crate::error::ApiError;
use sqlx::postgres::PgPool;

pub struct Robot;

impl Robot {
    pub async fn new(conn: &PgPool, robot_serial_number: &str) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
INSERT INTO robot (robot_serial_number)
VALUES ($1)
        "#,
            &robot_serial_number
        )
        .execute(conn)
        .await
        .map_err(|_| ApiError::RobotInitializationFailed)?;

        // By default a new robot will be in idle.
        Command::idle(conn, robot_serial_number).await?;

        Ok(())
    }
}
