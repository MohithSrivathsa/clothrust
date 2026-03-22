use actix_web::{web, HttpResponse};
use actix_session::Session;
use tera::Context;
use crate::AppState;
use crate::models::CartItem;
use crate::auth::current_user;

fn base_ctx(session: &Session) -> Context {
    let mut items: Vec<CartItem> = session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default();
    // Clear cart if it contains old SVG placeholder URLs
    if items.iter().any(|i| i.image_url.contains(".svg") || i.image_url.is_empty()) {
        items.clear();
        session.insert("cart", &items).ok();
    }
    let mut ctx = Context::new();
    ctx.insert("cart_count", &items.len());
    if let Some(user) = current_user(session) {
        ctx.insert("user", &user);
        ctx.insert("logged_in", &true);
    } else {
        ctx.insert("logged_in", &false);
    }
    ctx
}

fn query_products(db: &rusqlite::Connection, sql: &str) -> Vec<serde_json::Value> {
    let mut stmt = match db.prepare(sql) { Ok(s) => s, Err(_) => return vec![] };
    stmt.query_map([], |row| Ok(serde_json::json!({
        "id":          row.get::<_,i64>(0)?,
        "name":        row.get::<_,String>(1)?,
        "description": row.get::<_,String>(2)?,
        "price":       row.get::<_,f64>(3)?,
        "category":    row.get::<_,String>(4)?,
        "sizes":  serde_json::from_str::<Vec<String>>(&row.get::<_,String>(5)?).unwrap_or_default(),
        "colors": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(6)?).unwrap_or_default(),
        "image_url":   row.get::<_,String>(7)?,
        "badge":       row.get::<_,Option<String>>(8)?,
        "stock":       row.get::<_,i32>(9)?,
    }))).unwrap().filter_map(|r| r.ok()).collect()
}

pub async fn index(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let db = data.db.lock().unwrap();
    let products = query_products(&db, "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products LIMIT 4");
    ctx.insert("products", &products);
    ctx.insert("page", "home");
    let html = data.tera.render("index.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn shop(data: web::Data<AppState>, session: Session, query: web::Query<std::collections::HashMap<String,String>>) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let category = query.get("category").cloned().unwrap_or_default();
    let db = data.db.lock().unwrap();
    let sql = if category.is_empty() {
        "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products".to_string()
    } else {
        format!("SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products WHERE category='{}'", category)
    };
    let products = query_products(&db, &sql);
    ctx.insert("products", &products);
    ctx.insert("active_category", &category);
    ctx.insert("page", "shop");
    let html = data.tera.render("shop.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn product_detail(data: web::Data<AppState>, session: Session, path: web::Path<i64>) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    match db.query_row(
        "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products WHERE id=?1",
        [id], |row| Ok(serde_json::json!({
            "id": row.get::<_,i64>(0)?, "name": row.get::<_,String>(1)?,
            "description": row.get::<_,String>(2)?, "price": row.get::<_,f64>(3)?,
            "category": row.get::<_,String>(4)?,
            "sizes":  serde_json::from_str::<Vec<String>>(&row.get::<_,String>(5)?).unwrap_or_default(),
            "colors": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(6)?).unwrap_or_default(),
            "image_url": row.get::<_,String>(7)?, "badge": row.get::<_,Option<String>>(8)?,
            "stock": row.get::<_,i32>(9)?,
        }))
    ) {
        Ok(product) => {
            ctx.insert("product", &product);
            ctx.insert("page", "product");
            let html = data.tera.render("product.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
            HttpResponse::Ok().content_type("text/html").body(html)
        }
        Err(_) => HttpResponse::NotFound().body("Product not found"),
    }
}

pub async fn cart(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let items: Vec<CartItem> = session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default();
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    ctx.insert("cart", &serde_json::json!({"items": items, "total": total}));
    ctx.insert("page", "cart");
    let html = data.tera.render("cart.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn checkout(data: web::Data<AppState>, session: Session) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let items: Vec<CartItem> = session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default();
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    ctx.insert("cart", &serde_json::json!({"items": items, "total": total}));
    ctx.insert("page", "checkout");
    let html = data.tera.render("checkout.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn order_success(data: web::Data<AppState>, session: Session, query: web::Query<std::collections::HashMap<String,String>>) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    ctx.insert("order_id", &query.get("order_id").cloned().unwrap_or_default());
    ctx.insert("page", "success");
    let html = data.tera.render("success.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn admin(data: web::Data<AppState>, _session: Session) -> HttpResponse {
    let html = data.tera.render("admin.html", &Context::new()).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn search(data: web::Data<AppState>, session: Session, query: web::Query<std::collections::HashMap<String,String>>) -> HttpResponse {
    let mut ctx = base_ctx(&session);
    let q = query.get("q").cloned().unwrap_or_default();
    let db = data.db.lock().unwrap();
    let sql = format!(
        "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products WHERE name LIKE '%{0}%' OR category LIKE '%{0}%' OR description LIKE '%{0}%'", q
    );
    let products = query_products(&db, &sql);
    ctx.insert("products", &products);
    ctx.insert("query", &q);
    ctx.insert("page", "search");
    let html = data.tera.render("search.html", &ctx).unwrap_or_else(|e| format!("Error: {}", e));
    HttpResponse::Ok().content_type("text/html").body(html)
}
