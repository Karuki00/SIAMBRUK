CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS product_variants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku TEXT NOT NULL COLLATE NOCASE UNIQUE,
    name TEXT NOT NULL,
    purchase_price INTEGER NOT NULL CHECK (purchase_price >= 0),
    selling_price INTEGER NOT NULL CHECK (selling_price >= 0),
    current_stock INTEGER NOT NULL DEFAULT 0 CHECK (current_stock >= 0),
    minimum_stock INTEGER NOT NULL DEFAULT 0 CHECK (minimum_stock >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS purchases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reference TEXT,
    purchased_at TEXT NOT NULL,
    total_amount INTEGER NOT NULL DEFAULT 0 CHECK (total_amount >= 0),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS purchase_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_id INTEGER NOT NULL REFERENCES purchases(id) ON DELETE CASCADE,
    variant_id INTEGER NOT NULL REFERENCES product_variants(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_cost INTEGER NOT NULL CHECK (unit_cost >= 0),
    subtotal INTEGER NOT NULL CHECK (subtotal >= 0)
);

CREATE TABLE IF NOT EXISTS sales (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reference TEXT,
    sold_at TEXT NOT NULL,
    total_amount INTEGER NOT NULL DEFAULT 0 CHECK (total_amount >= 0),
    total_profit INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sale_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sale_id INTEGER NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    variant_id INTEGER NOT NULL REFERENCES product_variants(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price INTEGER NOT NULL CHECK (unit_price >= 0),
    unit_cost INTEGER NOT NULL CHECK (unit_cost >= 0),
    subtotal INTEGER NOT NULL CHECK (subtotal >= 0),
    profit INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS inventory_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    variant_id INTEGER NOT NULL REFERENCES product_variants(id),
    movement_type TEXT NOT NULL CHECK (movement_type IN ('purchase', 'sale', 'adjustment')),
    quantity INTEGER NOT NULL CHECK (quantity <> 0),
    stock_before INTEGER NOT NULL CHECK (stock_before >= 0),
    stock_after INTEGER NOT NULL CHECK (stock_after >= 0),
    reference TEXT,
    reason TEXT,
    moved_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS smart_criteria (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    weight REAL NOT NULL CHECK (weight >= 0),
    criterion_type TEXT NOT NULL CHECK (criterion_type IN ('benefit', 'cost')),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE IF NOT EXISTS smart_evaluations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    evaluated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    costing_method TEXT NOT NULL,
    total_weight REAL NOT NULL CHECK (total_weight > 0)
);

CREATE TABLE IF NOT EXISTS smart_evaluation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    evaluation_id INTEGER NOT NULL REFERENCES smart_evaluations(id) ON DELETE CASCADE,
    variant_id INTEGER NOT NULL REFERENCES product_variants(id),
    criterion_id INTEGER NOT NULL REFERENCES smart_criteria(id),
    raw_value REAL NOT NULL,
    utility_value REAL NOT NULL,
    weighted_contribution REAL NOT NULL,
    UNIQUE (evaluation_id, variant_id, criterion_id)
);

CREATE TABLE IF NOT EXISTS smart_ranking_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    evaluation_id INTEGER NOT NULL REFERENCES smart_evaluations(id) ON DELETE CASCADE,
    variant_id INTEGER NOT NULL REFERENCES product_variants(id),
    final_score REAL NOT NULL,
    ranking_position INTEGER NOT NULL,
    UNIQUE (evaluation_id, variant_id)
);

CREATE INDEX IF NOT EXISTS idx_variants_product ON product_variants(product_id);
CREATE INDEX IF NOT EXISTS idx_movements_variant_date ON inventory_movements(variant_id, moved_at);
CREATE INDEX IF NOT EXISTS idx_sales_date ON sales(sold_at);
CREATE INDEX IF NOT EXISTS idx_purchases_date ON purchases(purchased_at);

INSERT OR IGNORE INTO smart_criteria (code, name, weight, criterion_type) VALUES
    ('profit', 'Keuntungan', 0.30, 'benefit'),
    ('units_sold', 'Produk Terjual', 0.40, 'benefit'),
    ('stock', 'Stok', 0.15, 'cost'),
    ('purchase_price', 'Harga Pembelian', 0.15, 'cost');
