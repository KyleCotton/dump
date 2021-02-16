use crate::error::ApiError;
use chrono::{serde::ts_seconds, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

// TODO: Set this to a sensible value
const TIME_ISSUED_BUFFER: i64 = 1000;
const TIME_INSTRUCTION_BUFFER: i64 = 1000;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Command {
    command_id: i64,
    pub robot_serial_number: String,
    #[serde(with = "ts_seconds")]
    time_issued: chrono::DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    time_instruction: chrono::DateTime<Utc>,
    pub instruction: Instruction,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CleaningPattern {
    ZigZag,
    Circular,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AbortReason {
    LowBattery,
    Saftey,
    Obstacle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Instruction {
    Continue,
    Pause,
    Abort(AbortReason),
    Task(CleaningPattern),
    Idle,
}

impl Command {
    pub async fn new(
        conn: &PgPool,
        robot_serial_number: &str,
        time_issued: chrono::DateTime<Utc>,
        time_instruction: chrono::DateTime<Utc>,
        instruction: &Instruction,
    ) -> Result<Command, ApiError> {
        // Check that the commands was given within the
        //   time buffer
        let time_difference = (chrono::Utc::now() - time_issued).num_seconds().abs();
        if time_difference > TIME_ISSUED_BUFFER {
            println!(
                "Error: Outside of the time buffer\nTime Diff: {}",
                time_difference
            );
            return Err(ApiError::CommandNotInTimeIssuedBuffer);
        }

        let instruction_json = serde_json::to_string(instruction).map_err(|e| {
            println!("Instrution Json: {:?}", e);
            ApiError::SerializationError
        })?;

        let command_id = sqlx::query!(
            r#"
        INSERT INTO Commands (robot_serial_number, time_issued, time_instruction, instruction)
        VALUES ( $1, $2, $3, $4 )
        RETURNING command_id
                "#,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction_json
        )
        .fetch_one(conn)
        .await
        .map_err(|e| {
            println!("Command New: {:?}", e);
            ApiError::DatabaseConnFailed
        })?
        .command_id;

        let robot_serial_number = robot_serial_number.to_string();

        Ok(Self {
            command_id,
            robot_serial_number,
            time_issued,
            time_instruction,
            instruction: instruction.clone(),
            completed: false,
        })
    }

    // Get the current task the robot is doing
    pub async fn current(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        sqlx::query!(
            r#"
SELECT * FROM Commands C
NATURAL JOIN
(SELECT MAX(C1.time_issued) AS time_issued,
        $1 AS robot_serial_number
FROM Commands C1
WHERE C1.robot_serial_number = $1) MaxTimeIssued
               "#,
            robot_serial_number
        )
        .fetch_one(conn)
        .await
        .map(|cmd| Self {
            command_id: cmd.command_id,
            robot_serial_number: cmd.robot_serial_number,
            time_issued: cmd.time_issued,
            time_instruction: cmd.time_issued,
            instruction: serde_json::from_str(&cmd.instruction)
                .unwrap_or(Instruction::Abort(AbortReason::Saftey)),
            completed: cmd.completed,
        })
        .map_err(|e| {
            println!("Command Latest: {:?}", e);
            ApiError::DatabaseConnFailed
        })
    }

    /// Checks to see if there are any pending command for this robot
    pub async fn pending(conn: &PgPool, robot_serial_number: &str) -> Result<Command, ApiError> {
        let pending_commands = sqlx::query!(
            r#"
SELECT * FROM Commands C
WHERE C.robot_serial_number = $1 AND
      C.completed = false
ORDER BY C.time_instruction DESC
               "#,
            robot_serial_number
        )
        .fetch_all(conn)
        .await
        .map(|cmds| {
            let mut pending = Vec::new();

            for c in cmds {
                pending.push(Self {
                    command_id: c.command_id,
                    robot_serial_number: c.robot_serial_number,
                    time_issued: c.time_issued,
                    time_instruction: c.time_issued,
                    instruction: serde_json::from_str(&c.instruction)
                        .unwrap_or(Instruction::Abort(AbortReason::Saftey)),
                    completed: c.completed,
                })
            }

            println!("Pending Commands {:?}", pending);

            pending
        })
        .map_err(|_| ApiError::DatabaseConnFailed)?;

        match pending_commands.get(0) {
            Some(cmd) if cmd.valid_time_instruction() => Ok(cmd.clone()),
            _ => Command::idle(conn, robot_serial_number).await,
        }
    }

    pub async fn complete(&self, conn: &PgPool) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
UPDATE Commands C
SET completed = true
WHERE C.command_id= $1
               "#,
            self.command_id
        )
        .execute(conn)
        .await
        .map_err(|e| {
            println!("Command Latest: {:?}", e);
            ApiError::DatabaseConnFailed
        })?;

        Ok(())
    }

    pub fn valid_time_instruction(&self) -> bool {
        let time_difference = (chrono::Utc::now() - self.time_instruction)
            .num_seconds()
            .abs();

        time_difference < TIME_INSTRUCTION_BUFFER
    }
}

impl Command {
    // Abort the current task with the given reason
    pub async fn abort(
        conn: &PgPool,
        robot_serial_number: &str,
        reason: &AbortReason,
    ) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Abort(reason.clone()),
        )
        .await?)
    }

    // Idle task the current task with the given reason
    pub async fn idle(conn: &PgPool, robot_serial_number: &str) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Idle,
        )
        .await?)
    }

    pub async fn task(
        conn: &PgPool,
        robot_serial_number: &str,
        cleaning_pattern: &CleaningPattern,
    ) -> Result<Self, ApiError> {
        // Create a new command with the current time
        let time_now = chrono::Utc::now();

        Ok(Command::new(
            conn,
            robot_serial_number,
            time_now,
            time_now,
            &Instruction::Task(cleaning_pattern.clone()),
        )
        .await?)
    }
}
