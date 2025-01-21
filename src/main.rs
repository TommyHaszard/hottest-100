mod routes;
mod db;

#[macro_use] extern crate rocket;

use std::sync::Mutex;
use dotenv::dotenv;
use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::tokio::sync::OnceCell;
use sqlx::{ConnectOptions, Connection};
use sqlx_postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn init_pool() -> PgPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
    // let conn = PgConnectOptions::new()
    //     .host("localhost")
    //     .port(5433)
    //     .username("hottest_user")
    //     .password("hottest_pass")
    //     .database("hottest_db");
    //
    // let pool= PgPool::connect_with(conn).await.unwrap();
    // pool

}

// State for storing the access token
pub struct SpotifyState {
    token: Mutex<Option<String>>,
    user: Mutex<Option<String>>
}

#[launch]
fn rocket() -> _ {
    let access_token = SpotifyState {
        token: Mutex::new(None),
        user: Mutex::new(None)
    };

    rocket::build()
        .attach(AdHoc::on_ignite("Database Pool", |rocket| async {
            let pool = init_pool().await;
            DB_POOL.set(pool).unwrap();
            rocket }))
        .manage(access_token)
        .manage(Client::new())
        .mount("/", routes![routes::index, routes::callback, routes::main_page, routes::files, routes::search_songs])
        .mount("/main", FileServer::from("static"))
}