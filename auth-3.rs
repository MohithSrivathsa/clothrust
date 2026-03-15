use actix_web::{web, HttpResponse};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

// ── Auth0 config ──────────────────────────────────────────────
const AUTH0_DOMAIN:        &str = "dev-y3p5ee28fvb47hvn.us.auth0.com";
const AUTH0_CLIENT_ID:     &str = "oOGxXOHOUS6GqAWKacHbF2IT4tYVZeYS";
const AUTH0_CLIENT_SECRET: &str = "hRbpr6JrA8O56HvW7146iFIMhzjZRoFb0JS7-mcNzZc_bZcRI8aih5EtSe5WB4kM";
const REDIRECT_URI:        &str = "http://127.0.0.1:8080/callback";

// ── Types ─────────────────────────────────────────────────────
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
    pub code:  Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token:     Option<String>,
    #[allow(dead_code)]
    token_type:   String,
}

// ── /login ────────────────────────────────────────────────────
pub async fn login(session: Session) -> HttpResponse {
    // Generate a random state to prevent CSRF
    let state = Uuid::new_v4().to_string();
    let _ = session.insert("oauth_state", &state);

    let url = format!(
        "https://{}/authorize\
         ?response_type=code\
         &client_id={}\
         &redirect_uri={}\
         &scope=openid%20profile%20email\
         &state={}\
         &prompt=login",
        AUTH0_DOMAIN,
        AUTH0_CLIENT_ID,
        urlencoding::encode(REDIRECT_URI),
        state
    );
    HttpResponse::Found()
        .insert_header(("Location", url))
        .finish()
}

// ── /callback ─────────────────────────────────────────────────
pub async fn callback(
    query: web::Query<CallbackQuery>,
    session: Session,
) -> HttpResponse {
    log::info!("=== CALLBACK HIT === code={:?} error={:?}", query.code.is_some(), query.error);

    // Handle Auth0 errors
    if let Some(err) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown error");
        log::error!("Auth0 error: {} — {}", err, desc);
        return HttpResponse::Ok()
            .content_type("text/html")
            .body(format!("<h2>Auth Error</h2><p>{}: {}</p><a href='/'>Home</a>", err, desc));
    }

    let code = match &query.code {
        Some(c) => c.clone(),
        None => {
            log::error!("No code in callback");
            return HttpResponse::Ok()
                .content_type("text/html")
                .body("<h2>Error</h2><p>No auth code received</p><a href='/'>Home</a>");
        }
    };

    log::info!("Exchanging code for token at https://{}/oauth/token", AUTH0_DOMAIN);

    // Exchange code for token
    let client = reqwest::Client::new();
    let token_url = format!("https://{}/oauth/token", AUTH0_DOMAIN);

    let params = [
        ("grant_type",    "authorization_code"),
        ("client_id",     AUTH0_CLIENT_ID),
        ("client_secret", AUTH0_CLIENT_SECRET),
        ("code",          code.as_str()),
        ("redirect_uri",  REDIRECT_URI),
    ];

    let token_res = match client.post(&token_url).form(&params).send().await {
        Ok(r)  => r,
        Err(e) => {
            log::error!("Token exchange network error: {}", e);
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("<h2>Network Error</h2><p>{}</p><a href='/'>Home</a>", e));
        }
    };

    let status = token_res.status();
    log::info!("Token response status: {}", status);

    let token_text = token_res.text().await.unwrap_or_default();
    log::info!("Token response body: {}", &token_text[..token_text.len().min(300)]);

    let token: TokenResponse = match serde_json::from_str(&token_text) {
        Ok(t)  => t,
        Err(e) => {
            log::error!("Token parse failed: {} | body: {}", e, &token_text[..token_text.len().min(500)]);
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("<h2>Token Error</h2><pre>{}</pre><p>Parse: {}</p><a href='/'>Home</a>", token_text, e));
        }
    };

    log::info!("Got access token, fetching userinfo");

    // Fetch user info
    let userinfo_url = format!("https://{}/userinfo", AUTH0_DOMAIN);
    let user_res = match client
        .get(&userinfo_url)
        .bearer_auth(&token.access_token)
        .send().await
    {
        Ok(r)  => r,
        Err(e) => {
            log::error!("Userinfo network error: {}", e);
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("<h2>Userinfo Error</h2><p>{}</p><a href='/'>Home</a>", e));
        }
    };

    let user_text = user_res.text().await.unwrap_or_default();
    log::info!("Userinfo body: {}", &user_text[..user_text.len().min(300)]);

    let user: AuthUser = match serde_json::from_str(&user_text) {
        Ok(u)  => u,
        Err(e) => {
            log::error!("Userinfo parse failed: {} | body: {}", e, &user_text[..user_text.len().min(500)]);
            return HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("<h2>Userinfo Parse Error</h2><pre>{}</pre><p>{}</p><a href='/'>Home</a>", user_text, e));
        }
    };

    log::info!("Login success for: {:?}", user.email);

    // Save to session
    let _ = session.insert("user", &user);
    let _ = session.insert("access_token", &token.access_token);

    HttpResponse::Found()
        .insert_header(("Location", "/profile"))
        .finish()
}

// ── /logout ───────────────────────────────────────────────────
pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    let url = format!(
        "https://{}/v2/logout?client_id={}&returnTo={}",
        AUTH0_DOMAIN,
        AUTH0_CLIENT_ID,
        urlencoding::encode("http://127.0.0.1:8080/")
    );
    HttpResponse::Found()
        .insert_header(("Location", url))
        .finish()
}

// ── /profile ──────────────────────────────────────────────────
pub async fn profile(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let user: Option<AuthUser> = session.get("user").unwrap_or(None);

    match user {
        None => HttpResponse::Found()
            .insert_header(("Location", "/login"))
            .finish(),
        Some(u) => {
            // Fetch order history for this user
            let db = data.db.lock().unwrap();
            let orders: Vec<serde_json::Value> = {
                let email = u.email.clone().unwrap_or_default();
                match db.prepare(
                    "SELECT id, total, status, created_at, payment_method FROM orders WHERE email=?1 ORDER BY created_at DESC LIMIT 10"
                ) {
                    Ok(mut stmt) => {
                        stmt.query_map([email.as_str()], |row| {
                            Ok(serde_json::json!({
                                "id":             row.get::<_,String>(0)?,
                                "total":          row.get::<_,f64>(1)?,
                                "status":         row.get::<_,String>(2)?,
                                "created_at":     row.get::<_,String>(3)?,
                                "payment_method": row.get::<_,String>(4)?,
                            }))
                        })
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default()
                    }
                    Err(_) => vec![],
                }
            };
            drop(db);

            let cart_count: usize = {
                use crate::models::CartItem;
                session.get::<Vec<CartItem>>("cart")
                    .unwrap_or(None)
                    .unwrap_or_default()
                    .len()
            };

            let mut ctx = tera::Context::new();
            ctx.insert("user", &u);
            ctx.insert("orders", &orders);
            ctx.insert("cart_count", &cart_count);
            ctx.insert("page", "profile");

            let html = data.tera.render("profile.html", &ctx)
                .unwrap_or_else(|e| format!("Template error: {}", e));
            HttpResponse::Ok().content_type("text/html").body(html)
        }
    }
}

// ── Helper: get current user from session ────────────────────
pub fn current_user(session: &Session) -> Option<AuthUser> {
    session.get::<AuthUser>("user").unwrap_or(None)
}
