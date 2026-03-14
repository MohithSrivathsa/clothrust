use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub sizes: Vec<String>,
    pub colors: Vec<String>,
    pub image_url: String,
    pub badge: Option<String>,
    pub stock: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItem {
    pub product_id: i64,
    pub product_name: String,
    pub price: f64,
    pub size: String,
    pub color: String,
    pub quantity: i32,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
    pub total: f64,
}

impl Cart {
    pub fn calculate_total(&mut self) {
        self.total = self.items.iter().map(|i| i.price * i.quantity as f64).sum();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToCartRequest {
    pub product_id: i64,
    pub size: String,
    pub color: String,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFromCartRequest {
    pub product_id: i64,
    pub size: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCartRequest {
    pub product_id: i64,
    pub size: String,
    pub color: String,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequest {
    pub customer_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub pincode: String,
    pub payment_method: String,
    pub items: Vec<CartItem>,
    pub total: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub customer_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub pincode: String,
    pub payment_method: String,
    pub status: String,
    pub total: f64,
    pub created_at: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub product_name: String,
    pub size: String,
    pub color: String,
    pub quantity: i32,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse { success: true, data: Some(data), message: None }
    }
    pub fn error(msg: &str) -> ApiResponse<()> {
        ApiResponse { success: false, data: None, message: Some(msg.to_string()) }
    }
}
