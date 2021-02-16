use crate::command::Command;
use crate::command::Instruction::{Abort, Idle};
use crate::poll::Poll;

use sqlx::postgres::PgPool;
use std::env;

const TEST_SERIAL: &str = "testing1";

// fn setup_test() {
//     println!("SETTING UP TESTS");
// }

async fn db_connect() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be set");
    let database_pool = PgPool::connect(&database_url)
        .await
        .expect("to get database pool");

    database_pool
}

#[tokio::test]
async fn set_idle_poll() {
    let conn = &db_connect().await;
    assert_eq!(1, 1);
}

// spawn_app().await.expect("Failed to spawn our app.");

// Set the robot to the idle state
// Command::idle(conn, TEST_SERIAL).await.unwrap();

// let poll = Poll {
//     robot_serial_number: TEST_SERIAL.to_string(),
//     instruction: Idle,
//     battery_level: 90,
// };

// let result = Poll::poll(conn, &poll).await.unwrap();

// assert_eq!(Idle, result.instruction);
