# 🧵 ThreadCraft — Full Stack Rust E-Commerce Store

A complete, production-ready e-commerce web application for selling custom printed t-shirts and clothing, built with **Rust (Actix-Web)** + **HTML/CSS/JavaScript**.

---

## 🚀 Tech Stack

| Layer     | Technology |
|-----------|------------|
| Backend   | **Rust** with **Actix-Web 4** |
| Database  | **SQLite** via `rusqlite` (bundled) |
| Templates | **Tera** (Jinja2-like HTML templates) |
| Session   | Cookie-based sessions (Actix-Session) |
| Frontend  | Vanilla **HTML + CSS + JavaScript** |
| Fonts     | Playfair Display, DM Sans, Space Mono |

---

## 📁 Project Structure

```
tshirt-store/
├── Cargo.toml                  # Rust dependencies
├── src/
│   ├── main.rs                 # App entry point, routes
│   ├── models.rs               # Data structures
│   ├── db.rs                   # SQLite setup & seeding
│   └── handlers/
│       ├── mod.rs
│       ├── pages.rs            # HTML page handlers
│       └── api.rs              # JSON API handlers
├── templates/
│   ├── base.html               # Layout (nav, footer)
│   ├── index.html              # Homepage
│   ├── shop.html               # Product listing
│   ├── product.html            # Product detail
│   ├── cart.html               # Shopping cart
│   ├── checkout.html           # Checkout form
│   └── success.html            # Order confirmation
└── static/
    ├── css/main.css            # All styles
    └── js/main.js              # Frontend logic
```

---

## ⚙️ Setup & Run

### Prerequisites
- **Rust** (install from https://rustup.rs)
- No other dependencies needed — SQLite is bundled!

### Run in Development

```bash
# Clone or copy the project
cd tshirt-store

# Build and run (first build takes ~2 min to compile)
RUST_LOG=info cargo run

# The store will be available at:
# http://127.0.0.1:8080
```

### Build for Production

```bash
cargo build --release
./target/release/tshirt-store
```

---

## 🌐 Pages & Routes

### Web Pages
| Route | Description |
|-------|-------------|
| `GET /` | Homepage with featured products |
| `GET /shop` | All products with filters |
| `GET /shop?category=T-Shirts` | Filter by category |
| `GET /product/{id}` | Product detail page |
| `GET /cart` | Shopping cart |
| `GET /checkout` | Checkout form |
| `GET /order-success?order_id=...` | Order confirmation |

### REST API
| Method | Route | Description |
|--------|-------|-------------|
| `GET` | `/api/products` | List all products |
| `GET` | `/api/products/{id}` | Single product |
| `GET` | `/api/cart` | Get cart contents |
| `POST` | `/api/cart/add` | Add item to cart |
| `POST` | `/api/cart/remove` | Remove item |
| `POST` | `/api/cart/update` | Update quantity |
| `POST` | `/api/orders` | Place an order |
| `GET` | `/api/orders/{id}` | Get order details |

---

## 🛍️ Features

- ✅ Product catalog with categories (T-Shirts, Shirts, Hoodies)
- ✅ Product detail with size & color selection
- ✅ Shopping cart with session persistence
- ✅ Quantity updates and item removal
- ✅ Checkout with delivery address form
- ✅ Multiple payment methods (UPI, Card, COD, Net Banking)
- ✅ Order placement and confirmation page
- ✅ Price-based shipping (free above ₹999)
- ✅ Client-side sorting and price filtering
- ✅ Responsive design (mobile-friendly)
- ✅ Animated product cards & interactions
- ✅ Quick-add modal from product listings
- ✅ Toast notifications
- ✅ SQLite database with auto-seeded products

---

## 🎨 Design

- **Theme:** Luxury editorial dark — deep navy/charcoal backgrounds with red accent (#e94560)
- **Typography:** Playfair Display (headings) + DM Sans (body) + Space Mono (prices/labels)
- **Responsive:** Fully mobile-friendly with hamburger menu
- **Animations:** Intersection Observer scroll reveals, floating hero T-shirt, animated marquee

---

## 🔧 Customization

### Add products
Edit `src/db.rs` in the `seed_products()` function.

### Change colors/fonts
Edit `static/css/main.css` — all design tokens are CSS variables at the top:
```css
:root {
  --accent: #e94560;   /* Change brand color here */
  --bg: #0a0a0f;       /* Background */
  ...
}
```

### Add payment gateway
In `src/handlers/api.rs`, extend the `place_order` handler to integrate Razorpay, Stripe, or PayU SDKs.

---

## 📦 Adding Real Product Images

Replace the SVG placeholders by adding image files to `static/images/` and updating the `image_url` field in `src/db.rs`. The templates already reference these URLs.

---

## 🔒 Production Checklist

- [ ] Set `SECRET_KEY` as environment variable (not generated at runtime)
- [ ] Move from SQLite to PostgreSQL for production
- [ ] Add HTTPS (use nginx or Caddy as reverse proxy)
- [ ] Set `RUST_LOG=warn` in production
- [ ] Add rate limiting for API routes
- [ ] Integrate real payment gateway (Razorpay recommended for India)
- [ ] Add image upload for products

---

Made with ❤️ and Rust 🦀
