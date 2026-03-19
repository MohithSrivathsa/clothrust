use actix_web::{web, App, HttpServer, middleware};
use actix_files::Files;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use rusqlite::Connection;
use std::sync::Mutex;
use tera::Tera;

mod db;
mod models;
mod routes;
mod handlers;
mod auth;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub tera: Tera,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let conn = Connection::open("store.db").expect("Failed to open database");
    db::initialize_db(&conn).expect("Failed to initialize database");
    db::seed_products(&conn).expect("Failed to seed products");

    let data = web::Data::new(AppState {
        db: Mutex::new(conn),
        tera: Tera::new("templates/**/*").expect("Failed to load templates"),
    });

    let secret_key = Key::generate();
    log::info!("🚀 ThreadCraft Store running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
            .wrap(middleware::Logger::default())
            .service(Files::new("/static", "static"))
            // Auth
            .route("/login",    web::get().to(auth::login))
            .route("/callback", web::get().to(auth::callback))
            .route("/logout",   web::get().to(auth::logout))
            .route("/profile",  web::get().to(auth::profile))
            // Pages
            .route("/",              web::get().to(handlers::pages::index))
            .route("/shop",          web::get().to(handlers::pages::shop))
            .route("/product/{id}",  web::get().to(handlers::pages::product_detail))
            .route("/cart",          web::get().to(handlers::pages::cart))
            .route("/checkout",      web::get().to(handlers::pages::checkout))
            .route("/order-success", web::get().to(handlers::pages::order_success))
            .route("/admin",         web::get().to(handlers::pages::admin))
            .route("/search",        web::get().to(handlers::pages::search))
            // API
            .route("/api/products",        web::get().to(handlers::api::get_products))
            .route("/api/products/{id}",   web::get().to(handlers::api::get_product))
            .route("/api/cart/add",        web::post().to(handlers::api::add_to_cart))
            .route("/api/cart/remove",     web::post().to(handlers::api::remove_from_cart))
            .route("/api/cart/update",     web::post().to(handlers::api::update_cart))
            .route("/api/cart",            web::get().to(handlers::api::get_cart))
            .route("/api/orders",          web::post().to(handlers::api::place_order))
            .route("/api/orders/{id}",     web::get().to(handlers::api::get_order))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
