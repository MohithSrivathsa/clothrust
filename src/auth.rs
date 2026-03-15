use actix_web::{web, HttpResponse};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

const AUTH0_DOMAIN:        &str = "dev-y3p5ee28fvb47hvn.us.auth0.com";
const AUTH0_CLIENT_ID:     &str = "oOGxXOHOUS6GqAWKacHbF2IT4tYVZeYS";
const AUTH0_CLIENT_SECRET: &str = "hRbpr6JrA8O56HvW7146iFIMhzjZRoFb0JS7-mcNzZc_bZcRI8aih5EtSe5WB4kM";
const REDIRECT_URI:        &str = "http://127.0.0.1:8080/callback";

// ── Admin whitelist ───────────────────────────────────────────
const ADMIN_EMAILS: &[&str] = &[
    "mohithsrivathsa111@gmail.com",
];

pub fn is_admin(user: &AuthUser) -> bool {
    if let Some(email) = &user.email {
        ADMIN_EMAILS.contains(&email.as_str())
    } else {
        false
    }
}

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
struct TokenResponse { access_token: String }

pub async fn login(session: Session) -> HttpResponse {
    let state = Uuid::new_v4().to_string();
    let _ = session.insert("oauth_state", &state);
    let url = format!(
        "https://{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope=openid%20profile%20email&state={}",
        AUTH0_DOMAIN, AUTH0_CLIENT_ID, urlencoding::encode(REDIRECT_URI), state
    );
    HttpResponse::Found().insert_header(("Location", url)).finish()
}

pub async fn callback(query: web::Query<CallbackQuery>, session: Session) -> HttpResponse {
    log::info!("CALLBACK: code={} error={:?}", query.code.is_some(), query.error);
    if let Some(err) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("");
        return HttpResponse::Ok().content_type("text/html").body(
            format!("<h2>Auth Error: {}</h2><p>{}</p><a href='/'>Back</a>", err, desc)
        );
    }
    let code = match &query.code {
        Some(c) => c.clone(),
        None => return HttpResponse::Found().insert_header(("Location", "/")).finish(),
    };
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap();
    let token_url = format!("https://{}/oauth/token", AUTH0_DOMAIN);
    let params = [
        ("grant_type","authorization_code"),("client_id",AUTH0_CLIENT_ID),
        ("client_secret",AUTH0_CLIENT_SECRET),("code",code.as_str()),("redirect_uri",REDIRECT_URI),
    ];
    let token_body = match client.post(&token_url).form(&params).send().await {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(e) => return HttpResponse::Ok().content_type("text/html").body(format!("<h2>Network Error</h2><pre>{}</pre><a href='/'>Back</a>",e)),
    };
    log::info!("Token: {}", &token_body[..token_body.len().min(200)]);
    let token: TokenResponse = match serde_json::from_str(&token_body) {
        Ok(t) => t,
        Err(e) => return HttpResponse::Ok().content_type("text/html").body(format!("<h2>Token Error</h2><pre>{}</pre><p>{}</p><a href='/'>Back</a>",token_body,e)),
    };
    let userinfo_url = format!("https://{}/userinfo", AUTH0_DOMAIN);
    let user_body = match client.get(&userinfo_url).bearer_auth(&token.access_token).send().await {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(e) => return HttpResponse::Ok().content_type("text/html").body(format!("<h2>Userinfo Error</h2><pre>{}</pre><a href='/'>Back</a>",e)),
    };
    let user: AuthUser = match serde_json::from_str(&user_body) {
        Ok(u) => u,
        Err(e) => return HttpResponse::Ok().content_type("text/html").body(format!("<h2>Profile Error</h2><pre>{}</pre><p>{}</p><a href='/'>Back</a>",user_body,e)),
    };
    log::info!("Login success: {:?} admin={}", user.email, is_admin(&user));
    session.insert("user", &user).ok();
    let name = user.name.as_deref().unwrap_or("welcome").to_string();
    HttpResponse::Ok().content_type("text/html").body(format!(
        r#"<!DOCTYPE html><html><head><meta charset="UTF-8"/>
<style>body{{font-family:sans-serif;background:#fafaf8;display:flex;align-items:center;justify-content:center;height:100vh;flex-direction:column;gap:1rem}}
.s{{width:36px;height:36px;border:2px solid #e4e1da;border-top-color:#c8502a;border-radius:50%;animation:spin .8s linear infinite}}
@keyframes spin{{to{{transform:rotate(360deg)}}}}</style></head>
<body><div class="s"></div><p style="color:#6b6560">Signing in as {}…</p>
<script>setTimeout(()=>location.href='/profile',200)</script></body></html>"#, name
    ))
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    let url = format!(
        "https://{}/v2/logout?client_id={}&returnTo={}",
        AUTH0_DOMAIN, AUTH0_CLIENT_ID, urlencoding::encode("http://127.0.0.1:8080/")
    );
    HttpResponse::Found().insert_header(("Location", url)).finish()
}

pub async fn profile(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let user: Option<AuthUser> = session.get("user").unwrap_or(None);
    match user {
        None => HttpResponse::Found().insert_header(("Location", "/login")).finish(),
        Some(u) => {
            let orders: Vec<serde_json::Value> = {
                let db = data.db.lock().unwrap();
                let email = u.email.clone().unwrap_or_default();
                let x = match db.prepare(
                    "SELECT id,total,status,created_at,payment_method FROM orders WHERE email=?1 ORDER BY created_at DESC LIMIT 10"
                ) {
                    Ok(mut stmt) => stmt.query_map([email.as_str()], |row| Ok(serde_json::json!({
                        "id": row.get::<_,String>(0)?, "total": row.get::<_,f64>(1)?,
                        "status": row.get::<_,String>(2)?, "created_at": row.get::<_,String>(3)?,
                        "payment_method": row.get::<_,String>(4)?,
                    }))).map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default(),
                    Err(_) => vec![],
                }; x
            };
            let cart_count: usize = {
                use crate::models::CartItem;
                session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default().len()
            };
            let admin = is_admin(&u);
            let mut ctx = tera::Context::new();
            ctx.insert("user", &u);
            ctx.insert("orders", &orders);
            ctx.insert("cart_count", &cart_count);
            ctx.insert("logged_in", &true);
            ctx.insert("is_admin", &admin);
            ctx.insert("page", "profile");
            let html = match data.tera.render("profile.html", &ctx) {
                Ok(h) => h,
                Err(e) => { log::error!("Profile template error: {:?}", e); format!("<h2>Template Error</h2><pre>{}</pre><a href='/'>Back</a>",e) }
            };
            HttpResponse::Ok().content_type("text/html").body(html)
        }
    }
}

pub async fn admin_guard(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let user: Option<AuthUser> = session.get("user").unwrap_or(None);
    match user {
        None => HttpResponse::Found().insert_header(("Location", "/login")).finish(),
        Some(u) if !is_admin(&u) => {
            HttpResponse::Forbidden().content_type("text/html").body(
                r#"<!DOCTYPE html><html><head><meta charset="UTF-8"/>
<style>body{font-family:'Jost',sans-serif;background:#fafaf8;display:flex;align-items:center;justify-content:center;height:100vh;flex-direction:column;gap:1rem;color:#1a1814}
h1{font-size:4rem;margin:0}p{color:#6b6560}a{color:#c8502a}</style></head>
<body><h1>403</h1><p>You don't have admin access.</p><a href="/">← Back to store</a></body></html>"#
            )
        }
        Some(_) => {
            let html = data.tera.render("admin.html", &tera::Context::new())
                .unwrap_or_else(|e| format!("Template error: {}", e));
            HttpResponse::Ok().content_type("text/html").body(html)
        }
    }
}

pub fn current_user(session: &Session) -> Option<AuthUser> {
    session.get::<AuthUser>("user").unwrap_or(None)
}
