use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::command::{
    AbortReason, Command, Instruction,
    Instruction::{Abort, Idle, Task},
};
use crate::error::ApiError;

const MINIMUM_BATTERY_LEVEL: i64 = 50;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Poll {
    pub robot_serial_number: String,
    pub instruction: Instruction,
    pub battery_level: i64,
}

impl Poll {
    pub async fn poll(conn: &PgPool, next_command: &Self) -> Result<Command, ApiError> {
        // Check the battery of the robot
        if !next_command.check_battery().await {
            return Ok(Command::abort(
                conn,
                &next_command.robot_serial_number,
                &AbortReason::LowBattery,
            )
            .await?);
        }

        // Get the previous command the robot was doing
        let prev_command = Command::current(conn, &next_command.robot_serial_number).await?;

        println!("Latest Command: {:?}", prev_command);

        // Determine the response based on the robots state
        match (&prev_command.instruction, &next_command.instruction) {
            // If the robot has said it needs to abort the task is completed,
            // and the robot will abort
            (_, Abort(reason)) => {
                prev_command.complete(conn).await.ok();
                Command::abort(conn, &next_command.robot_serial_number, reason).await
            }

            // If the old task is the same as the new one, keep doing it.
            (Task(prev), Task(new)) if prev == new => Ok(prev_command),

            // The previous task completed, mark it as complete and look for other tasks
            (Task(_), Idle) => {
                prev_command.complete(conn).await.ok();
                Command::pending(conn, &prev_command.robot_serial_number).await
            }

            // If we are now idle, check for pending commands, otherwise stay idle
            (_, Idle) => Command::pending(conn, &prev_command.robot_serial_number).await,

            // Any other instructions order is not supported
            _unsupported_instruction => Err(ApiError::CmdInstructionNotSupported),
        }
    }

    /// Checks the current battery level of the Robot
    ///
    /// If the battery level is not sufficent the robot will
    /// be told to abort due to low battery.
    async fn check_battery(&self) -> bool {
        self.battery_level >= 0
            && self.battery_level > MINIMUM_BATTERY_LEVEL
            && self.battery_level <= 100
    }
}
