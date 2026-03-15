use actix_web::{web, HttpResponse};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::models::CartItem;

fn stripe_secret() -> String {
    std::env::var("STRIPE_SECRET_KEY").unwrap_or_else(|_| {
        log::warn!("STRIPE_SECRET_KEY not set in environment");
        String::new()
    })
}
fn stripe_pk() -> String {
    std::env::var("STRIPE_PUBLISHABLE_KEY").unwrap_or_else(|_| {
        log::warn!("STRIPE_PUBLISHABLE_KEY not set in environment");
        String::new()
    })
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentReq {
    pub amount:   u64,   // in paise (₹ × 100)
    pub currency: Option<String>,
    pub customer_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntentResponse {
    pub client_secret: String,
    pub payment_intent_id: String,
    pub amount: u64,
}

/// POST /api/stripe/create-payment-intent
pub async fn create_payment_intent(
    req: web::Json<CreatePaymentIntentReq>,
    session: Session,
) -> HttpResponse {
    let items: Vec<CartItem> = session
        .get::<Vec<CartItem>>("cart")
        .unwrap_or(None)
        .unwrap_or_default();

    if items.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cart is empty"
        }));
    }

    // Calculate total from cart (don't trust client-sent amount)
    let total_paise: u64 = items.iter()
        .map(|i| (i.price * i.quantity as f64 * 100.0) as u64)
        .sum();

    // Minimum ₹1 = 100 paise
    if total_paise < 100 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Amount too small"
        }));
    }

    let client = reqwest::Client::new();

    // Build form params for Stripe API
    let mut params = vec![
        ("amount".to_string(), total_paise.to_string()),
        ("currency".to_string(), "inr".to_string()),
        ("payment_method_types[]".to_string(), "card".to_string()),
        ("description".to_string(), "ThreadCraft Order".to_string()),
    ];

    // Add customer email as metadata if available
    if let Some(email) = &req.customer_email {
        params.push(("receipt_email".to_string(), email.clone()));
        params.push(("metadata[customer_email]".to_string(), email.clone()));
    }

    // Add cart summary as metadata
    let item_summary: Vec<String> = items.iter()
        .map(|i| format!("{}×{} ({})", i.quantity, i.product_name, i.size))
        .collect();
    params.push(("metadata[items]".to_string(), item_summary.join(", ")));

    log::info!("Creating Stripe PaymentIntent for ₹{}", total_paise / 100);

    let res = match client
        .post("https://api.stripe.com/v1/payment_intents")
        .basic_auth(stripe_secret().as_str(), Some(""))
        .form(&params)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log::error!("Stripe request failed: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to connect to payment provider"
            }));
        }
    };

    let body = res.text().await.unwrap_or_default();
    log::info!("Stripe response: {}", &body[..body.len().min(300)]);

    let parsed: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Invalid response from payment provider"
        })),
    };

    if let Some(err) = parsed.get("error") {
        log::error!("Stripe error: {}", err);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": err["message"].as_str().unwrap_or("Payment setup failed")
        }));
    }

    let client_secret = parsed["client_secret"].as_str().unwrap_or("").to_string();
    let pi_id = parsed["id"].as_str().unwrap_or("").to_string();

    if client_secret.is_empty() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "No client secret returned"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "client_secret": client_secret,
        "payment_intent_id": pi_id,
        "amount": total_paise,
        "publishable_key": stripe_pk(),
    }))
}

/// GET /api/stripe/config — returns PK to frontend safely
pub async fn stripe_config() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "publishable_key": stripe_pk()
    }))
}
