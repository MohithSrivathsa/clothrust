use rusqlite::{Connection, Result};

pub fn initialize_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            category TEXT NOT NULL,
            sizes TEXT NOT NULL,
            colors TEXT NOT NULL,
            image_url TEXT NOT NULL,
            badge TEXT,
            stock INTEGER NOT NULL DEFAULT 100
        );
        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            customer_name TEXT NOT NULL,
            email TEXT NOT NULL,
            phone TEXT NOT NULL,
            address TEXT NOT NULL,
            city TEXT NOT NULL,
            state TEXT NOT NULL,
            pincode TEXT NOT NULL,
            payment_method TEXT NOT NULL,
            stripe_payment_id TEXT,
            status TEXT NOT NULL DEFAULT 'confirmed',
            total REAL NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS carts (
            user_sub TEXT NOT NULL,
            cart_json TEXT NOT NULL DEFAULT '[]',
            updated_at TEXT NOT NULL,
            PRIMARY KEY(user_sub)
        );
        CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id TEXT NOT NULL,
            product_name TEXT NOT NULL,
            size TEXT NOT NULL,
            color TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            FOREIGN KEY(order_id) REFERENCES orders(id)
        );
        CREATE TABLE IF NOT EXISTS saved_carts (
            email TEXT PRIMARY KEY,
            cart_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
    ")?;
    Ok(())
}

pub fn seed_products(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))?;
    if count > 0 { return Ok(()); }

    let products = vec![
        ("Geometric Sunrise Tee","Hand-printed geometric sunrise design using eco-friendly inks.",799.0,"T-Shirts",r#"["XS","S","M","L","XL","XXL"]"#,r#"["White","Black","Sand","Sky Blue"]"#,"/static/images/tee1.svg",Some("Best Seller")),
        ("Abstract Waves Oversized Tee","Oversized fit with fluid abstract wave print. Premium 240gsm cotton.",999.0,"T-Shirts",r#"["S","M","L","XL","XXL"]"#,r#"["Off White","Charcoal","Sage Green"]"#,"/static/images/tee2.svg",Some("New")),
        ("Mandala Spirit Shirt","Intricate mandala print with deep symbolic meaning.",1199.0,"Shirts",r#"["XS","S","M","L","XL"]"#,r#"["White","Navy","Burgundy"]"#,"/static/images/shirt1.svg",None),
        ("Urban Sketch Hoodie","Architectural sketch print on premium fleece hoodie.",1899.0,"Hoodies",r#"["S","M","L","XL","XXL"]"#,r#"["Grey Melange","Black","Cream"]"#,"/static/images/hoodie1.svg",Some("Limited")),
        ("Botanical Print Tee","Delicate botanical illustration print. Soft-washed vintage feel.",849.0,"T-Shirts",r#"["XS","S","M","L","XL"]"#,r#"["White","Peach","Mint"]"#,"/static/images/tee3.svg",None),
        ("Typography Drop Shoulder","Bold typographic art print on drop-shoulder silhouette.",1099.0,"T-Shirts",r#"["S","M","L","XL","XXL"]"#,r#"["Black","White","Olive"]"#,"/static/images/tee4.svg",Some("Trending")),
        ("Celestial Map Shirt","Detailed celestial map print on lightweight poplin.",1349.0,"Shirts",r#"["XS","S","M","L","XL"]"#,r#"["Navy","Black","Stone"]"#,"/static/images/shirt2.svg",None),
        ("Vintage Varsity Hoodie","Retro varsity-inspired graphic on heavyweight hoodie.",2199.0,"Hoodies",r#"["S","M","L","XL","XXL"]"#,r#"["Maroon","Forest Green","Navy"]"#,"/static/images/hoodie2.svg",Some("Premium")),
    ];

    for (name, desc, price, cat, sizes, colors, img, badge) in products {
        conn.execute(
            "INSERT INTO products (name,description,price,category,sizes,colors,image_url,badge,stock) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params![name, desc, price, cat, sizes, colors, img, badge, 100],
        )?;
    }
    Ok(())
}

/// Save cart to DB for a logged-in user
pub fn save_cart(conn: &Connection, email: &str, cart_json: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO saved_carts (email, cart_json, updated_at) VALUES (?1,?2,?3)
         ON CONFLICT(email) DO UPDATE SET cart_json=excluded.cart_json, updated_at=excluded.updated_at",
        rusqlite::params![email, cart_json, now],
    )?;
    Ok(())
}

/// Load saved cart from DB for a logged-in user
pub fn load_cart(conn: &Connection, email: &str) -> Option<String> {
    conn.query_row(
        "SELECT cart_json FROM saved_carts WHERE email=?1",
        [email],
        |r| r.get(0),
    ).ok()
}

/// Clear saved cart after order is placed
pub fn clear_cart(conn: &Connection, email: &str) -> Result<()> {
    conn.execute("DELETE FROM saved_carts WHERE email=?1", [email])?;
    Ok(())
}
