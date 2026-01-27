mod api;

#[macro_use] extern crate rocket;

use dotenv::dotenv;
use reqwest::Client;
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::tokio::sync::OnceCell;
use sqlx::{ConnectOptions, Connection};
use sqlx_postgres::{PgPool, PgPoolOptions};
use crate::api::{external_api, internal_api};

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn init_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")

}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let static_dir = std::env::var("STATIC_DIR").expect("STATIC_DIR must be set");

    let figment = rocket::Config::figment()
        .merge(("port", 8080))
        .merge(("address", "0.0.0.0"));
    rocket::custom(figment)
        .attach(AdHoc::on_ignite("Database Pool", |rocket| async {
            let pool = init_pool().await;
            DB_POOL.set(pool).unwrap();
            rocket }))
        .manage(Client::new())
        .mount("/", routes![internal_api::index, internal_api::login_page, external_api::callback, internal_api::main_page, internal_api::files, internal_api::search_songs, internal_api::save_songs, internal_api::get_songs, internal_api::generate_playlist, internal_api::get_music_taste])
        .mount("/main", FileServer::from(static_dir))

}
