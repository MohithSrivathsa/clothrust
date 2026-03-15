use actix_web::{web, HttpResponse};
use actix_session::Session;
use uuid::Uuid;
use chrono::Utc;
use crate::AppState;
use crate::models::*;

// ── Cart helper ───────────────────────────────────────────────
fn get_cart_items(session: &Session) -> Vec<CartItem> {
    session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default()
}

// ── GET /api/products ─────────────────────────────────────────
pub async fn get_products(data: web::Data<AppState>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products ORDER BY id"
    ).unwrap();
    let products: Vec<serde_json::Value> = stmt.query_map([], |r| Ok(serde_json::json!({
        "id": r.get::<_,i64>(0)?,
        "name": r.get::<_,String>(1)?,
        "description": r.get::<_,String>(2)?,
        "price": r.get::<_,f64>(3)?,
        "category": r.get::<_,String>(4)?,
        "sizes": serde_json::from_str::<Vec<String>>(&r.get::<_,String>(5)?).unwrap_or_default(),
        "colors": serde_json::from_str::<Vec<String>>(&r.get::<_,String>(6)?).unwrap_or_default(),
        "image_url": r.get::<_,String>(7)?,
        "badge": r.get::<_,Option<String>>(8)?,
        "stock": r.get::<_,i32>(9)?,
    }))).unwrap().filter_map(|r| r.ok()).collect();
    HttpResponse::Ok().json(serde_json::json!({"success": true, "data": products}))
}

// ── GET /api/products/{id} ────────────────────────────────────
pub async fn get_product(data: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    match db.query_row(
        "SELECT id,name,description,price,category,sizes,colors,image_url,badge,stock FROM products WHERE id=?1",
        [id], |r| Ok(serde_json::json!({
            "id": r.get::<_,i64>(0)?,
            "name": r.get::<_,String>(1)?,
            "price": r.get::<_,f64>(3)?,
            "sizes": serde_json::from_str::<Vec<String>>(&r.get::<_,String>(5)?).unwrap_or_default(),
            "colors": serde_json::from_str::<Vec<String>>(&r.get::<_,String>(6)?).unwrap_or_default(),
            "image_url": r.get::<_,String>(7)?,
            "badge": r.get::<_,Option<String>>(8)?,
            "stock": r.get::<_,i32>(9)?,
        }))
    ) {
        Ok(p) => HttpResponse::Ok().json(serde_json::json!({"success": true, "data": p})),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({"success": false})),
    }
}

// ── GET /api/cart ─────────────────────────────────────────────
pub async fn get_cart(session: Session) -> HttpResponse {
    let items = get_cart_items(&session);
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    HttpResponse::Ok().json(serde_json::json!({"success": true, "data": {"items": items, "total": total}}))
}

// ── POST /api/cart/add ────────────────────────────────────────
pub async fn add_to_cart(data: web::Data<AppState>, session: Session, req: web::Json<AddToCartRequest>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    match db.query_row(
        "SELECT id,name,price,image_url,stock FROM products WHERE id=?1",
        [req.product_id], |r| Ok((
            r.get::<_,i64>(0)?, r.get::<_,String>(1)?,
            r.get::<_,f64>(2)?, r.get::<_,String>(3)?,
            r.get::<_,i32>(4)?,
        ))
    ) {
        Ok((id, name, price, image_url, stock)) => {
            if stock <= 0 {
                return HttpResponse::Ok().json(serde_json::json!({"success": false, "error": "Out of stock"}));
            }
            drop(db);
            let mut items = get_cart_items(&session);
            if let Some(existing) = items.iter_mut().find(|i| {
                i.product_id == id && i.size == req.size && i.color == req.color
            }) {
                existing.quantity += req.quantity;
            } else {
                items.push(CartItem { product_id: id, product_name: name, price, size: req.size.clone(), color: req.color.clone(), quantity: req.quantity, image_url });
            }
            let count = items.len();
            session.insert("cart", &items).ok();
            HttpResponse::Ok().json(serde_json::json!({"success": true, "cart_count": count}))
        }
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({"success": false})),
    }
}

// ── POST /api/cart/remove ─────────────────────────────────────
pub async fn remove_from_cart(session: Session, req: web::Json<RemoveFromCartRequest>) -> HttpResponse {
    let mut items = get_cart_items(&session);
    items.retain(|i| !(i.product_id == req.product_id && i.size == req.size && i.color == req.color));
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    let count = items.len();
    session.insert("cart", &items).ok();
    HttpResponse::Ok().json(serde_json::json!({"success": true, "total": total, "cart_count": count}))
}

// ── POST /api/cart/update ─────────────────────────────────────
pub async fn update_cart(session: Session, req: web::Json<UpdateCartRequest>) -> HttpResponse {
    let mut items = get_cart_items(&session);
    if req.quantity == 0 {
        items.retain(|i| !(i.product_id == req.product_id && i.size == req.size && i.color == req.color));
    } else if let Some(item) = items.iter_mut().find(|i| {
        i.product_id == req.product_id && i.size == req.size && i.color == req.color
    }) {
        item.quantity = req.quantity;
    }
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    let count = items.len();
    session.insert("cart", &items).ok();
    HttpResponse::Ok().json(serde_json::json!({"success": true, "total": total, "cart_count": count}))
}

// ── POST /api/orders ──────────────────────────────────────────
pub async fn place_order(data: web::Data<AppState>, session: Session, req: web::Json<OrderRequest>) -> HttpResponse {
    let order_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let db = data.db.lock().unwrap();

    if let Err(e) = db.execute(
        "INSERT INTO orders (id,customer_name,email,phone,address,city,state,pincode,payment_method,status,total,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![order_id, req.customer_name, req.email, req.phone, req.address, req.city, req.state, req.pincode, req.payment_method, "confirmed", req.total, created_at]
    ) {
        log::error!("Order insert failed: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
    }

    for item in &req.items {
        db.execute(
            "INSERT INTO order_items (order_id,product_name,size,color,quantity,price) VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![order_id, item.product_name, item.size, item.color, item.quantity, item.price]
        ).ok();
        // Auto-decrement stock
        db.execute(
            "UPDATE products SET stock = MAX(stock - ?1, 0) WHERE name = ?2",
            rusqlite::params![item.quantity, item.product_name]
        ).ok();
    }

    drop(db);
    session.insert("cart", Vec::<CartItem>::new()).ok();
    HttpResponse::Ok().json(serde_json::json!({"success": true, "order_id": order_id}))
}

// ── GET /api/orders/{id} ──────────────────────────────────────
pub async fn get_order(data: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    match db.query_row(
        "SELECT id,customer_name,email,status,total,created_at FROM orders WHERE id=?1",
        [&id], |r| Ok(serde_json::json!({
            "id": r.get::<_,String>(0)?,
            "customer_name": r.get::<_,String>(1)?,
            "email": r.get::<_,String>(2)?,
            "status": r.get::<_,String>(3)?,
            "total": r.get::<_,f64>(4)?,
            "created_at": r.get::<_,String>(5)?,
        }))
    ) {
        Ok(o) => HttpResponse::Ok().json(serde_json::json!({"success": true, "data": o})),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({"success": false})),
    }
}

// ── POST /api/admin/stock/{id} ────────────────────────────────
#[derive(serde::Deserialize)]
pub struct StockUpdate { pub stock: i32 }

pub async fn update_stock(data: web::Data<AppState>, path: web::Path<i64>, body: web::Json<StockUpdate>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    match db.execute("UPDATE products SET stock=?1 WHERE id=?2", rusqlite::params![body.stock, id]) {
        Ok(_) => {
            log::info!("Stock updated: product {} → {}", id, body.stock);
            HttpResponse::Ok().json(serde_json::json!({"success": true, "stock": body.stock}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

// ── POST /api/admin/orders/{id}/status ───────────────────────
#[derive(serde::Deserialize)]
pub struct StatusUpdate { pub status: String }

pub async fn update_order_status(data: web::Data<AppState>, path: web::Path<String>, body: web::Json<StatusUpdate>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    db.execute("UPDATE orders SET status=?1 WHERE id=?2", rusqlite::params![body.status, id]).ok();
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}

// ── GET /api/admin/stats ──────────────────────────────────────
pub async fn admin_stats(data: web::Data<AppState>) -> HttpResponse {
    let db = data.db.lock().unwrap();

    let order_count: i64 = db.query_row("SELECT COUNT(*) FROM orders", [], |r| r.get(0)).unwrap_or(0);
    let revenue: f64     = db.query_row("SELECT COALESCE(SUM(total),0) FROM orders", [], |r| r.get(0)).unwrap_or(0.0);
    let product_count: i64 = db.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let low_stock: i64   = db.query_row("SELECT COUNT(*) FROM products WHERE stock < 10", [], |r| r.get(0)).unwrap_or(0);

    let mut ostmt = db.prepare(
        "SELECT id,customer_name,email,total,status,payment_method,created_at FROM orders ORDER BY created_at DESC LIMIT 20"
    ).unwrap();
    let orders: Vec<serde_json::Value> = ostmt.query_map([], |r| Ok(serde_json::json!({
        "id": r.get::<_,String>(0)?, "customer_name": r.get::<_,String>(1)?,
        "email": r.get::<_,String>(2)?, "total": r.get::<_,f64>(3)?,
        "status": r.get::<_,String>(4)?, "payment_method": r.get::<_,String>(5)?,
        "created_at": r.get::<_,String>(6)?,
    }))).unwrap().filter_map(|r| r.ok()).collect();

    let mut pstmt = db.prepare(
        "SELECT id,name,category,price,stock,badge FROM products ORDER BY stock ASC"
    ).unwrap();
    let products: Vec<serde_json::Value> = pstmt.query_map([], |r| Ok(serde_json::json!({
        "id": r.get::<_,i64>(0)?, "name": r.get::<_,String>(1)?,
        "category": r.get::<_,String>(2)?, "price": r.get::<_,f64>(3)?,
        "stock": r.get::<_,i32>(4)?, "badge": r.get::<_,Option<String>>(5)?,
    }))).unwrap().filter_map(|r| r.ok()).collect();

    HttpResponse::Ok().json(serde_json::json!({
        "order_count": order_count,
        "revenue": revenue,
        "product_count": product_count,
        "low_stock_count": low_stock,
        "orders": orders,
        "products": products,
    }))
}
