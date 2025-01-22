mod routes;
mod db;

#[macro_use] extern crate rocket;

use dotenv::dotenv;
use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::tokio::sync::OnceCell;
use sqlx::{ConnectOptions, Connection};
use sqlx_postgres::{PgPool, PgPoolOptions};

static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn init_pool() -> PgPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")

}

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment()
        .merge(("port", 8080))
        .merge(("address", "0.0.0.0"));
    rocket::custom(figment)
        .attach(AdHoc::on_ignite("Database Pool", |rocket| async {
            let pool = init_pool().await;
            DB_POOL.set(pool).unwrap();
            rocket }))
        .manage(Client::new())
        .mount("/", routes![routes::index, routes::callback, routes::main_page, routes::files, routes::search_songs, routes::save_songs, routes::get_songs])
        .mount("/main", FileServer::from("/app/static"))
}