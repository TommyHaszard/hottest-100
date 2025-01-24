mod api;

#[macro_use] extern crate rocket;

use dotenv::dotenv;
use reqwest::Client;
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::http::uri::Origin;
use rocket::Request;
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

pub struct RedirectFairing;

#[rocket::async_trait]
impl Fairing for RedirectFairing {
    fn info(&self) -> Info {
        Info {
            name: "Redirect All Refreshes to Root",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &'_ mut rocket::Data<'_>) {
        let original_path = request.uri().path();
        // Allow root (`/`) to pass through
        if original_path != "/" {
            // Replace request URI with `/` (root)
            request.set_uri(Origin::parse("/").unwrap());
        }
    }
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
        .mount("/", routes![external_api::index, external_api::callback, internal_api::main_page, internal_api::files, external_api::search_songs, internal_api::save_songs, internal_api::get_songs])
        .mount("/main", FileServer::from(static_dir))
        // .attach(RedirectFairing) // dirty solution to auto reauth if needed by redirecting to / on every refresh, probs better to redirect to / when theres an invalid access token

}