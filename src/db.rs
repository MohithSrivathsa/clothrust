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
            status TEXT NOT NULL DEFAULT 'confirmed',
            total REAL NOT NULL,
            created_at TEXT NOT NULL
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
    ")?;
    Ok(())
}

pub fn seed_products(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))?;
    if count > 0 { return Ok(()); }

    let products: Vec<(&str,&str,f64,&str,&str,&str,&str,Option<&str>)> = vec![
        (
            "Hanuman — The Devoted Warrior",
            "Hand-printed Hanuman artwork on premium 220gsm cotton. A symbol of strength, devotion and fearlessness. Eco-friendly water-based inks.",
            999.0, "T-Shirts",
            r#"["XS","S","M","L","XL","XXL"]"#,
            r#"["White","Sand","Cream"]"#,
            "/static/images/hanuman.webp",
            Some("Best Seller")
        ),
        (
            "Durga — The Invincible",
            "Goddess Durga in fierce warrior form. Bold HD print on 240gsm oversized tee. Power and grace in every thread.",
            1099.0, "T-Shirts",
            r#"["XS","S","M","L","XL","XXL"]"#,
            r#"["White","Off White","Light Grey"]"#,
            "/static/images/durga.webp",
            Some("New")
        ),
        (
            "Kali — Dark Mother",
            "The fierce and transformative Goddess Kali. Striking print that commands attention. Premium drop-shoulder fit.",
            1099.0, "T-Shirts",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Cream","Stone"]"#,
            "/static/images/kali.webp",
            None
        ),
        (
            "Arjuna & Krishna — Kurukshetra",
            "The legendary moment on the battlefield of Kurukshetra. Epic print on breathable cotton poplin shirt.",
            1349.0, "Shirts",
            r#"["XS","S","M","L","XL"]"#,
            r#"["White","Light Blue","Sand"]"#,
            "/static/images/arjuna-krishna.webp",
            Some("Limited")
        ),
        (
            "Poseidon — Lord of the Seas",
            "Greek god of the ocean in full fury. Heavyweight 260gsm hoodie with chest print. Built for legends.",
            1899.0, "Hoodies",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Grey Melange","Stone Blue"]"#,
            "/static/images/poseidon.webp",
            None
        ),
        (
            "Athena — Goddess of Wisdom",
            "Athena in battle armour — wisdom meets power. Precision printed on structured cotton shirt.",
            1249.0, "Shirts",
            r#"["XS","S","M","L","XL"]"#,
            r#"["White","Ivory","Light Grey"]"#,
            "/static/images/athena.webp",
            Some("Trending")
        ),
        (
            "Hades — King of the Underworld",
            "The dark and brooding ruler of the underworld. Oversized tee with dramatic artwork. Not for the faint-hearted.",
            999.0, "T-Shirts",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Off White","Cream"]"#,
            "/static/images/hades.webp",
            None
        ),
        (
            "Anubis — Guardian of the Dead",
            "Egyptian god Anubis in ceremonial stance. Ancient meets contemporary on premium fleece hoodie.",
            1999.0, "Hoodies",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Cream","Sand"]"#,
            "/static/images/anubis.webp",
            Some("Premium")
        ),
        (
            "Quetzalcoatl — The Feathered Serpent",
            "Aztec deity Quetzalcoatl in magnificent form. Vibrant HD print on 220gsm relaxed fit tee.",
            1099.0, "T-Shirts",
            r#"["XS","S","M","L","XL","XXL"]"#,
            r#"["White","Off White","Light Yellow"]"#,
            "/static/images/quetzalcoatl.webp",
            None
        ),
        (
            "Quetzalcoatl II — Serpent God",
            "Second variant of the Aztec feathered serpent — different composition, same raw power.",
            1099.0, "T-Shirts",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Cream","Stone"]"#,
            "/static/images/quetzalcoatl-2.webp",
            None
        ),
        (
            "Nezha — Lotus Prince",
            "Chinese mythological hero Nezha in dynamic action pose. Oversized streetwear tee with bold front print.",
            1149.0, "T-Shirts",
            r#"["XS","S","M","L","XL","XXL"]"#,
            r#"["White","Ivory","Light Pink"]"#,
            "/static/images/nezha.webp",
            Some("New")
        ),
        (
            "Susanoo — The Storm God",
            "Japanese god of storms Susanoo in legendary battle. Heavyweight premium hoodie for those who carry thunder.",
            1999.0, "Hoodies",
            r#"["S","M","L","XL","XXL"]"#,
            r#"["White","Grey Melange","Cream"]"#,
            "/static/images/susanoo.webp",
            Some("Limited")
        ),
        (
            "Thor — God of Thunder",
            "The Norse thunder god in full might. Classic fit shirt with intricate hand-finished print detail.",
            1249.0, "Shirts",
            r#"["XS","S","M","L","XL"]"#,
            r#"["White","Light Grey","Stone"]"#,
            "/static/images/thor.webp",
            Some("Trending")
        ),
    ];

    for (name, desc, price, cat, sizes, colors, img, badge) in products {
        conn.execute(
            "INSERT INTO products (name,description,price,category,sizes,colors,image_url,badge,stock) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            rusqlite::params![name, desc, price, cat, sizes, colors, img, badge, 100],
        )?;
    }
    Ok(())
}
