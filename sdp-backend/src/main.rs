use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use sqlx::postgres::PgPool;
use std::env;

mod api;
mod auth;
mod command;
mod error;
mod integration_test;
mod poll;
mod robot;
mod user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // Default port for the webserver is 8080, this can be overwritten
    // by an envirnment variable.
    // TODO: Test this works
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL to be set");
    let database_pool = PgPool::connect(&database_url)
        .await
        .expect("to get database pool");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default().allow_any_origin())
            .wrap(Logger::default())
            .data(database_pool.clone())
            .service(actix_files::Files::new("/static", "static/").show_files_listing())
            .service(api::user::create_user)
            .service(api::command::create_command)
            .service(api::poll::robot_poll)
            .service(api::auth::auth)
    })
    .bind(address)?
    .run()
    .await
}
