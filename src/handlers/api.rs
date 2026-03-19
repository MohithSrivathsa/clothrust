use actix_web::{web, HttpResponse};
use actix_session::Session;
use uuid::Uuid;
use chrono::Utc;
use crate::AppState;
use crate::models::*;

fn get_cart_items(session: &Session) -> Vec<CartItem> {
    session.get::<Vec<CartItem>>("cart").unwrap_or(None).unwrap_or_default()
}

pub async fn get_products(data: web::Data<AppState>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, name, description, price, category, sizes, colors, image_url, badge, stock FROM products"
    ).unwrap();
    let products: Vec<serde_json::Value> = stmt.query_map([], |row: &rusqlite::Row| {
        Ok(serde_json::json!({
            "id": row.get::<_,i64>(0)?, "name": row.get::<_,String>(1)?,
            "description": row.get::<_,String>(2)?, "price": row.get::<_,f64>(3)?,
            "category": row.get::<_,String>(4)?,
            "sizes": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(5)?).unwrap_or_default(),
            "colors": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(6)?).unwrap_or_default(),
            "image_url": row.get::<_,String>(7)?, "badge": row.get::<_,Option<String>>(8)?,
            "stock": row.get::<_,i32>(9)?,
        }))
    }).unwrap().filter_map(|r: Result<_, _>| r.ok()).collect();
    HttpResponse::Ok().json(ApiResponse::ok(products))
}

pub async fn get_product(data: web::Data<AppState>, path: web::Path<i64>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let id = path.into_inner();
    let result = db.query_row(
        "SELECT id, name, description, price, category, sizes, colors, image_url, badge, stock FROM products WHERE id=?1",
        [id],
        |row: &rusqlite::Row| Ok(serde_json::json!({
            "id": row.get::<_,i64>(0)?, "name": row.get::<_,String>(1)?,
            "description": row.get::<_,String>(2)?, "price": row.get::<_,f64>(3)?,
            "sizes": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(5)?).unwrap_or_default(),
            "colors": serde_json::from_str::<Vec<String>>(&row.get::<_,String>(6)?).unwrap_or_default(),
            "image_url": row.get::<_,String>(7)?, "badge": row.get::<_,Option<String>>(8)?,
        }))
    );
    match result {
        Ok(p) => HttpResponse::Ok().json(ApiResponse::ok(p)),
        Err(_) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Product not found")),
    }
}

pub async fn get_cart(session: Session) -> HttpResponse {
    let items = get_cart_items(&session);
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    HttpResponse::Ok().json(ApiResponse::ok(Cart { items, total }))
}

pub async fn add_to_cart(session: Session, data: web::Data<AppState>, req: web::Json<AddToCartRequest>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let product = db.query_row(
        "SELECT id, name, price, image_url FROM products WHERE id=?1",
        [req.product_id],
        |row: &rusqlite::Row| Ok((row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,f64>(2)?, row.get::<_,String>(3)?))
    );
    match product {
        Ok((id, name, price, image_url)) => {
            let mut items = get_cart_items(&session);
            if let Some(existing) = items.iter_mut().find(|i| i.product_id == id && i.size == req.size && i.color == req.color) {
                existing.quantity += req.quantity;
            } else {
                items.push(CartItem { product_id: id, product_name: name, price, size: req.size.clone(), color: req.color.clone(), quantity: req.quantity, image_url });
            }
            session.insert("cart", &items).unwrap();
            let count = items.len();
            HttpResponse::Ok().json(serde_json::json!({ "success": true, "cart_count": count }))
        }
        Err(_) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Product not found")),
    }
}

pub async fn remove_from_cart(session: Session, req: web::Json<RemoveFromCartRequest>) -> HttpResponse {
    let mut items = get_cart_items(&session);
    items.retain(|i| !(i.product_id == req.product_id && i.size == req.size && i.color == req.color));
    session.insert("cart", &items).unwrap();
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    HttpResponse::Ok().json(serde_json::json!({ "success": true, "total": total, "cart_count": items.len() }))
}

pub async fn update_cart(session: Session, req: web::Json<UpdateCartRequest>) -> HttpResponse {
    let mut items = get_cart_items(&session);
    if req.quantity == 0 {
        items.retain(|i| !(i.product_id == req.product_id && i.size == req.size && i.color == req.color));
    } else if let Some(item) = items.iter_mut().find(|i| i.product_id == req.product_id && i.size == req.size && i.color == req.color) {
        item.quantity = req.quantity;
    }
    session.insert("cart", &items).unwrap();
    let total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();
    HttpResponse::Ok().json(serde_json::json!({ "success": true, "total": total, "cart_count": items.len() }))
}

pub async fn place_order(data: web::Data<AppState>, session: Session, req: web::Json<OrderRequest>) -> HttpResponse {
    let order_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let db = data.db.lock().unwrap();

    let result = db.execute(
        "INSERT INTO orders (id, customer_name, email, phone, address, city, state, pincode, payment_method, status, total, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![order_id, req.customer_name, req.email, req.phone, req.address, req.city, req.state, req.pincode, req.payment_method, "confirmed", req.total, created_at]
    );
    if result.is_err() {
        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to place order"));
    }
    for item in &req.items {
        let _ = db.execute(
            "INSERT INTO order_items (order_id, product_name, size, color, quantity, price) VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![order_id, item.product_name, item.size, item.color, item.quantity, item.price]
        );
    }
    session.insert("cart", Vec::<CartItem>::new()).unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "success": true, "order_id": order_id }))
}

pub async fn get_order(data: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let db = data.db.lock().unwrap();
    let order_id = path.into_inner();
    let result = db.query_row(
        "SELECT id, customer_name, email, status, total, created_at FROM orders WHERE id=?1",
        [&order_id],
        |row: &rusqlite::Row| Ok(serde_json::json!({
            "id": row.get::<_,String>(0)?, "customer_name": row.get::<_,String>(1)?,
            "email": row.get::<_,String>(2)?, "status": row.get::<_,String>(3)?,
            "total": row.get::<_,f64>(4)?, "created_at": row.get::<_,String>(5)?,
        }))
    );
    match result {
        Ok(o) => HttpResponse::Ok().json(ApiResponse::ok(o)),
        Err(_) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Order not found")),
    }
}
