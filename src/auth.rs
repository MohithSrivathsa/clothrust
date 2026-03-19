use actix_web::{web, HttpResponse};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

const AUTH0_DOMAIN:        &str = "dev-y3p5ee28fvb47hvn.us.auth0.com";
const AUTH0_CLIENT_ID:     &str = "oOGxXOHOUS6GqAWKacHbF2IT4tYVZeYS";
const AUTH0_CLIENT_SECRET: &str = "hRbpr6JrA8O56HvW7146iFIMhzjZRoFb0JS7-mcNzZc_bZcRI8aih5EtSe5WB4kM";
const REDIRECT_URI:        &str = "http://127.0.0.1:8080/callback";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub sub:      String,
    pub name:     Option<String>,
    pub email:    Option<String>,
    pub picture:  Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code:              Option<String>,
    pub error:             Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub async fn login(session: Session) -> HttpResponse {
    let state = Uuid::new_v4().to_string();
    let _ = session.insert("oauth_state", &state);
    let url = format!(
        "https://{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid%20profile%20email&state={}",
        AUTH0_DOMAIN, AUTH0_CLIENT_ID,
        urlencoding::encode(REDIRECT_URI),
        state
    );
    HttpResponse::Found().insert_header(("Location", url)).finish()
}

pub async fn callback(query: web::Query<CallbackQuery>, session: Session) -> HttpResponse {
    log::info!("=== CALLBACK HIT === code={} error={:?}", query.code.is_some(), query.error);

    if let Some(err) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown");
        return HttpResponse::Ok().content_type("text/html")
            .body(format!("<h2>Auth Error</h2><p><b>{}</b>: {}</p><a href='/'>← Home</a>", err, desc));
    }

    let code = match &query.code {
        Some(c) => c.clone(),
        None => return HttpResponse::Ok().content_type("text/html")
            .body("<h2>Error</h2><p>No auth code received from Auth0</p><a href='/'>← Home</a>"),
    };

    let client = reqwest::Client::new();

    // Exchange code → token
    let token_res = client
        .post(format!("https://{}/oauth/token", AUTH0_DOMAIN))
        .form(&[
            ("grant_type",    "authorization_code"),
            ("client_id",     AUTH0_CLIENT_ID),
            ("client_secret", AUTH0_CLIENT_SECRET),
            ("code",          code.as_str()),
            ("redirect_uri",  REDIRECT_URI),
        ])
        .send().await;

    let token_res = match token_res {
        Ok(r) => r,
        Err(e) => return HttpResponse::Ok().content_type("text/html")
            .body(format!("<h2>Network Error</h2><p>{}</p><a href='/'>← Home</a>", e)),
    };

    let token_body = token_res.text().await.unwrap_or_default();
    log::info!("Token body: {}", &token_body[..token_body.len().min(400)]);

    let token: TokenResponse = match serde_json::from_str(&token_body) {
        Ok(t) => t,
        Err(e) => return HttpResponse::Ok().content_type("text/html")
            .body(format!("<h2>Token Error</h2><pre>{}</pre><p>{}</p><a href='/'>← Home</a>", token_body, e)),
    };

    // Fetch userinfo
    let user_res = client
        .get(format!("https://{}/userinfo", AUTH0_DOMAIN))
        .bearer_auth(&token.access_token)
        .send().await;

    let user_res = match user_res {
        Ok(r) => r,
        Err(e) => return HttpResponse::Ok().content_type("text/html")
            .body(format!("<h2>Userinfo Error</h2><p>{}</p><a href='/'>← Home</a>", e)),
    };

    let user_body = user_res.text().await.unwrap_or_default();
    log::info!("Userinfo: {}", &user_body[..user_body.len().min(300)]);

    let user: AuthUser = match serde_json::from_str(&user_body) {
        Ok(u) => u,
        Err(e) => return HttpResponse::Ok().content_type("text/html")
            .body(format!("<h2>Userinfo Parse Error</h2><pre>{}</pre><p>{}</p><a href='/'>← Home</a>", user_body, e)),
    };

    log::info!("✅ Login success: {:?}", user.email);
    let _ = session.insert("user", &user);
    let _ = session.insert("access_token", &token.access_token);

    HttpResponse::Found().insert_header(("Location", "/profile")).finish()
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    let url = format!(
        "https://{}/v2/logout?client_id={}&returnTo={}",
        AUTH0_DOMAIN, AUTH0_CLIENT_ID,
        urlencoding::encode("http://127.0.0.1:8080/")
    );
    HttpResponse::Found().insert_header(("Location", url)).finish()
}

pub async fn profile(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let user: Option<AuthUser> = session.get("user").unwrap_or(None);
    match user {
        None => HttpResponse::Found().insert_header(("Location", "/login")).finish(),
        Some(u) => {
            let db = data.db.lock().unwrap();
            let email = u.email.clone().unwrap_or_default();
            let orders: Vec<serde_json::Value> = match db.prepare(
                "SELECT id, total, status, created_at, payment_method FROM orders WHERE email=?1 ORDER BY created_at DESC LIMIT 10"
            ) {
                Ok(mut stmt) => stmt.query_map([email.as_str()], |row: &rusqlite::Row| {
                    Ok(serde_json::json!({
                        "id":             row.get::<_,String>(0)?,
                        "total":          row.get::<_,f64>(1)?,
                        "status":         row.get::<_,String>(2)?,
                        "created_at":     row.get::<_,String>(3)?,
                        "payment_method": row.get::<_,String>(4)?,
                    }))
                }).map(|rows| rows.filter_map(|r: Result<_, _>| r.ok()).collect()).unwrap_or_default(),
                Err(_) => vec![],
            };
            drop(db);

            use crate::models::CartItem;
            let cart_count: usize = session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default().len();

            let mut ctx = tera::Context::new();
            ctx.insert("user", &u);
            ctx.insert("orders", &orders);
            ctx.insert("cart_count", &cart_count);
            ctx.insert("logged_in", &true);
            ctx.insert("page", "profile");

            let html = data.tera.render("profile.html", &ctx)
                .unwrap_or_else(|e| format!("Template error: {}", e));
            HttpResponse::Ok().content_type("text/html").body(html)
        }
    }
}

pub fn current_user(session: &Session) -> Option<AuthUser> {
    session.get::<AuthUser>("user").unwrap_or(None)
}
