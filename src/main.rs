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
mod stripe;

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

    let secret_key = Key::from(&[
        0x4a,0x8f,0x2e,0x1c,0x9b,0x3d,0x7a,0x6e,
        0x5f,0x0c,0x8d,0x4b,0x2a,0x9c,0x1e,0x7f,
        0x3b,0x6a,0x0d,0x5e,0x8c,0x2f,0x4d,0x9a,
        0x1b,0x7c,0x3e,0x6f,0x0a,0x5d,0x8b,0x2c,
        0x9d,0x1f,0x4e,0x7b,0x3c,0x6d,0x0e,0x5f,
        0x8a,0x2b,0x4c,0x9e,0x1a,0x7d,0x3f,0x6e,
        0x0b,0x5c,0x8e,0x2d,0x4f,0x9b,0x1c,0x7e,
        0x3a,0x6c,0x0f,0x5b,0x8d,0x2e,0x4a,0x9f,
    ]);

    log::info!("🚀 ThreadCraft Store running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_http_only(true)
                    .build()
            )
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
            .route("/admin",         web::get().to(auth::admin_guard))
            .route("/search",        web::get().to(handlers::pages::search))
            // Cart API
            .route("/api/cart",            web::get().to(handlers::api::get_cart))
            .route("/api/cart/add",        web::post().to(handlers::api::add_to_cart))
            .route("/api/cart/remove",     web::post().to(handlers::api::remove_from_cart))
            .route("/api/cart/update",     web::post().to(handlers::api::update_cart))
            // Product API
            .route("/api/products",        web::get().to(handlers::api::get_products))
            .route("/api/products/{id}",   web::get().to(handlers::api::get_product))
            // Order API
            .route("/api/orders",          web::post().to(handlers::api::place_order))
            .route("/api/orders/{id}",     web::get().to(handlers::api::get_order))
            // Admin API
            .route("/api/admin/stats",                    web::get().to(handlers::api::admin_stats))
            .route("/api/admin/stock/{id}",               web::post().to(handlers::api::update_stock))
            .route("/api/admin/orders/{id}/status",       web::post().to(handlers::api::update_order_status))
            // Stripe API
            .route("/api/stripe/config",                  web::get().to(stripe::stripe_config))
            .route("/api/stripe/create-payment-intent",   web::post().to(stripe::create_payment_intent))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
