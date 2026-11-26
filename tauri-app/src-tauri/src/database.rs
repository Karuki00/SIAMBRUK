use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const INITIAL_MIGRATION: &str = include_str!("../migrations/001_initial.sql");
const INVENTORY_VALUE_MIGRATION: &str = include_str!("../migrations/002_inventory_value.sql");

pub struct Database {
    connection: Mutex<Connection>,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    pub ready: bool,
    pub schema_version: i64,
    pub product_count: i64,
    pub variant_count: i64,
    pub category_count: i64,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub product_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct NewProduct {
    pub name: String,
    pub category_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub id: i64,
    pub name: String,
    pub category_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductRow {
    pub id: i64,
    pub name: String,
    pub category_name: Option<String>,
    pub description: Option<String>,
    pub variant_count: i64,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewVariant {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub minimum_stock: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariant {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub minimum_stock: i64,
}

#[derive(Debug, Serialize)]
pub struct VariantRow {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub sku: String,
    pub name: String,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub current_stock: i64,
    pub minimum_stock: i64,
}

#[derive(Debug, Deserialize)]
pub struct StockAdjustment {
    pub variant_id: i64,
    pub quantity: i64,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseItemInput {
    pub variant_id: i64,
    pub quantity: i64,
    pub unit_cost: i64,
}

#[derive(Debug, Deserialize)]
pub struct NewPurchase {
    pub reference: Option<String>,
    pub purchased_at: String,
    pub notes: Option<String>,
    pub items: Vec<PurchaseItemInput>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResult {
    pub id: i64,
    pub total_amount: i64,
    pub total_profit: i64,
}

#[derive(Debug, Deserialize)]
pub struct SaleItemInput {
    pub variant_id: i64,
    pub quantity: i64,
    pub unit_price: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct NewSale {
    pub reference: Option<String>,
    pub sold_at: String,
    pub notes: Option<String>,
    pub items: Vec<SaleItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct SmartEvaluationRequest {
    pub period_start: String,
    pub period_end: String,
    pub costing_method: String,
}

#[derive(Debug, Serialize)]
pub struct SmartEvaluationResponse {
    pub evaluation_id: i64,
    pub period_start: String,
    pub period_end: String,
    pub results: Vec<crate::smart::SmartResult>,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub active_products: i64,
    pub variant_count: i64,
    pub total_stock: i64,
    pub sales_today: i64,
    pub revenue_today: i64,
    pub profit_today: i64,
    pub low_stock_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReportRangeRequest {
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Deserialize)]
pub struct VariantReportRequest {
    pub variant_id: i64,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Serialize)]
pub struct ReportPoint {
    pub date: String,
    pub units_sold: i64,
    pub revenue: i64,
    pub profit: i64,
}

#[derive(Debug, Serialize)]
pub struct VariantReportStats {
    pub variant_id: i64,
    pub product_name: String,
    pub variant_name: String,
    pub current_stock: i64,
    pub units_sold: i64,
    pub revenue: i64,
    pub profit: i64,
    pub points: Vec<ReportPoint>,
}

#[derive(Debug, Serialize)]
pub struct SmartHistoryRow {
    pub id: i64,
    pub period_start: String,
    pub period_end: String,
    pub evaluated_at: String,
    pub costing_method: String,
    pub ranking_count: i64,
}

#[derive(Debug, Serialize)]
pub struct MovementRow {
    pub id: i64,
    pub product_name: String,
    pub variant_name: String,
    pub movement_type: String,
    pub quantity: i64,
    pub stock_after: i64,
    pub moved_at: String,
    pub reason: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaleRow {
    pub id: i64,
    pub reference: Option<String>,
    pub sold_at: String,
    pub total_amount: i64,
    pub total_profit: i64,
}

#[derive(Debug, Serialize)]
pub struct PurchaseRow {
    pub id: i64,
    pub reference: Option<String>,
    pub purchased_at: String,
    pub total_amount: i64,
}

impl Database {
    pub fn open(app_data_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(app_data_dir)
            .map_err(|error| format!("Unable to create app data directory: {error}"))?;
        let path = app_data_dir.join("siambruk.sqlite3");
        let connection =
            Connection::open(&path).map_err(|error| format!("Unable to open database: {error}"))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| format!("Unable to configure database: {error}"))?;
        Self::migrate(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
            path,
        })
    }

    fn migrate(connection: &Connection) -> Result<(), String> {
        connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);")
            .map_err(|error| format!("Unable to create migration table: {error}"))?;
        let applied: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Unable to read migration version: {error}"))?;
        if applied < 1 {
            let transaction = connection
                .unchecked_transaction()
                .map_err(|error| format!("Unable to start migration: {error}"))?;
            transaction
                .execute_batch(INITIAL_MIGRATION)
                .map_err(|error| format!("Unable to apply initial migration: {error}"))?;
            transaction
                .execute("INSERT INTO schema_migrations (version) VALUES (1)", [])
                .map_err(|error| format!("Unable to record migration: {error}"))?;
            transaction
                .commit()
                .map_err(|error| format!("Unable to commit migration: {error}"))?;
        }
        if applied < 2 {
            let transaction = connection
                .unchecked_transaction()
                .map_err(|error| format!("Unable to start inventory value migration: {error}"))?;
            transaction
                .execute_batch(INVENTORY_VALUE_MIGRATION)
                .map_err(|error| format!("Unable to apply inventory value migration: {error}"))?;
            transaction
                .execute("INSERT INTO schema_migrations (version) VALUES (2)", [])
                .map_err(|error| format!("Unable to record inventory value migration: {error}"))?;
            transaction
                .commit()
                .map_err(|error| format!("Unable to commit inventory value migration: {error}"))?;
        }
        Ok(())
    }

    pub fn status(&self) -> Result<DatabaseStatus, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let count = |table: &str| -> Result<i64, String> {
            let sql = format!("SELECT COUNT(*) FROM {table}");
            connection
                .query_row(&sql, [], |row| row.get(0))
                .map_err(|error| format!("Unable to read {table} count: {error}"))
        };
        let schema_version = connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Unable to read schema version: {error}"))?;
        Ok(DatabaseStatus {
            ready: true,
            schema_version,
            product_count: count("products")?,
            variant_count: count("product_variants")?,
            category_count: count("categories")?,
            path: self.path.display().to_string(),
        })
    }

    pub fn categories(&self) -> Result<Vec<CategoryRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT c.id, c.name, c.description, COUNT(p.id)
                 FROM categories c LEFT JOIN products p ON p.category_id = c.id
                 GROUP BY c.id ORDER BY c.name COLLATE NOCASE",
            )
            .map_err(|error| format!("Unable to prepare categories query: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(CategoryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    product_count: row.get(3)?,
                })
            })
            .map_err(|error| format!("Unable to read categories: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map categories: {error}"))
    }

    pub fn create_category(&self, input: NewCategory) -> Result<CategoryRow, String> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err("Nama kategori wajib diisi".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        connection
            .execute(
                "INSERT INTO categories (name, description) VALUES (?1, ?2)",
                rusqlite::params![
                    name,
                    input
                        .description
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                ],
            )
            .map_err(|error| match error {
                rusqlite::Error::SqliteFailure(_, _) => "Nama kategori sudah digunakan".to_string(),
                other => format!("Unable to create category: {other}"),
            })?;
        let id = connection.last_insert_rowid();
        Ok(CategoryRow {
            id,
            name: name.to_string(),
            description: input.description,
            product_count: 0,
        })
    }

    pub fn delete_category(&self, category_id: i64) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let product_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM products WHERE category_id = ?1",
                [category_id],
                |row| row.get(0),
            )
            .map_err(|_| "Kategori tidak ditemukan".to_string())?;
        if product_count > 0 {
            return Err("Kategori masih digunakan oleh produk dan tidak dapat dihapus".to_string());
        }
        let changed = connection
            .execute("DELETE FROM categories WHERE id = ?1", [category_id])
            .map_err(|error| format!("Unable to delete category: {error}"))?;
        if changed == 0 {
            return Err("Kategori tidak ditemukan".to_string());
        }
        Ok(())
    }

    pub fn update_category(&self, input: UpdateCategory) -> Result<(), String> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err("Nama kategori wajib diisi".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        connection.execute("UPDATE categories SET name = ?1, description = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3", rusqlite::params![name, input.description, input.id]).map_err(|error| format!("Unable to update category: {error}"))?;
        Ok(())
    }

    pub fn products(&self) -> Result<Vec<ProductRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT p.id, p.name, c.name, p.description, COUNT(v.id), p.is_active
                 FROM products p LEFT JOIN categories c ON c.id = p.category_id
                 LEFT JOIN product_variants v ON v.product_id = p.id AND v.is_active = 1
                 WHERE p.is_active = 1
                 GROUP BY p.id ORDER BY p.name COLLATE NOCASE",
            )
            .map_err(|error| format!("Unable to prepare products query: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(ProductRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_name: row.get(2)?,
                    description: row.get(3)?,
                    variant_count: row.get(4)?,
                    is_active: row.get::<_, i64>(5)? == 1,
                })
            })
            .map_err(|error| format!("Unable to read products: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map products: {error}"))
    }

    pub fn create_product(&self, input: NewProduct) -> Result<ProductRow, String> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err("Nama produk wajib diisi".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        connection
            .execute(
                "INSERT INTO products (category_id, name, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    input.category_id,
                    name,
                    input
                        .description
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                ],
            )
            .map_err(|error| format!("Unable to create product: {error}"))?;
        Ok(ProductRow {
            id: connection.last_insert_rowid(),
            name: name.to_string(),
            category_name: None,
            description: input.description,
            variant_count: 0,
            is_active: true,
        })
    }

    pub fn delete_product(&self, product_id: i64) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let stock_total: i64 = connection
            .query_row(
                "SELECT COALESCE(SUM(current_stock), 0) FROM product_variants WHERE product_id = ?1 AND is_active = 1",
                [product_id],
                |row| row.get(0),
            )
            .map_err(|_| "Produk tidak ditemukan".to_string())?;
        let exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM products WHERE id = ?1 AND is_active = 1",
                [product_id],
                |row| row.get(0),
            )
            .map_err(|_| "Produk tidak ditemukan".to_string())?;
        if exists == 0 {
            return Err("Produk tidak ditemukan".to_string());
        }
        if stock_total > 0 {
            return Err("Produk tidak dapat dihapus karena masih memiliki stok. Kurangi stok terlebih dahulu.".to_string());
        }
        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Unable to start product deletion: {error}"))?;
        transaction
            .execute("UPDATE product_variants SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE product_id = ?1", [product_id])
            .map_err(|error| format!("Unable to deactivate product variants: {error}"))?;
        let changed = transaction
            .execute(
                "UPDATE products SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                [product_id],
            )
            .map_err(|error| format!("Unable to delete product: {error}"))?;
        if changed == 0 {
            return Err("Produk tidak ditemukan".to_string());
        }
        transaction
            .commit()
            .map_err(|error| format!("Unable to commit product deletion: {error}"))?;
        Ok(())
    }

    pub fn update_product(&self, input: UpdateProduct) -> Result<(), String> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err("Nama produk wajib diisi".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        connection.execute("UPDATE products SET name = ?1, category_id = ?2, description = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4", rusqlite::params![name, input.category_id, input.description, input.id]).map_err(|error| format!("Unable to update product: {error}"))?;
        Ok(())
    }

    pub fn variants(&self, product_id: Option<i64>) -> Result<Vec<VariantRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT v.id, v.product_id, p.name, v.sku, v.name, v.purchase_price,
                        v.selling_price, v.current_stock, v.minimum_stock
                 FROM product_variants v JOIN products p ON p.id = v.product_id
                 WHERE v.is_active = 1 AND p.is_active = 1 AND (?1 IS NULL OR v.product_id = ?1)
                 ORDER BY p.name COLLATE NOCASE, v.name COLLATE NOCASE",
            )
            .map_err(|error| format!("Unable to prepare variants query: {error}"))?;
        let rows = statement
            .query_map(rusqlite::params![product_id], |row| {
                Ok(VariantRow {
                    id: row.get(0)?,
                    product_id: row.get(1)?,
                    product_name: row.get(2)?,
                    sku: row.get(3)?,
                    name: row.get(4)?,
                    purchase_price: row.get(5)?,
                    selling_price: row.get(6)?,
                    current_stock: row.get(7)?,
                    minimum_stock: row.get(8)?,
                })
            })
            .map_err(|error| format!("Unable to read variants: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map variants: {error}"))
    }

    pub fn create_variant(&self, input: NewVariant) -> Result<VariantRow, String> {
        let sku = input.sku.trim();
        let name = input.name.trim();
        if sku.is_empty() || name.is_empty() {
            return Err("SKU dan nama varian wajib diisi".to_string());
        }
        if input.purchase_price < 0 || input.selling_price < 0 || input.minimum_stock < 0 {
            return Err("Harga dan batas stok tidak boleh negatif".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let product_name: String = connection
            .query_row(
                "SELECT name FROM products WHERE id = ?1 AND is_active = 1",
                [input.product_id],
                |row| row.get(0),
            )
            .map_err(|_| "Produk tidak ditemukan atau tidak aktif".to_string())?;
        connection
            .execute(
                "INSERT INTO product_variants (product_id, sku, name, purchase_price, selling_price, minimum_stock) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![input.product_id, sku, name, input.purchase_price, input.selling_price, input.minimum_stock],
            )
            .map_err(|error| match error {
                rusqlite::Error::SqliteFailure(_, _) => "SKU sudah digunakan".to_string(),
                other => format!("Unable to create variant: {other}"),
            })?;
        Ok(VariantRow {
            id: connection.last_insert_rowid(),
            product_id: input.product_id,
            product_name,
            sku: sku.to_string(),
            name: name.to_string(),
            purchase_price: input.purchase_price,
            selling_price: input.selling_price,
            current_stock: 0,
            minimum_stock: input.minimum_stock,
        })
    }

    pub fn update_variant(&self, input: UpdateVariant) -> Result<VariantRow, String> {
        let sku = input.sku.trim();
        let name = input.name.trim();
        if sku.is_empty() || name.is_empty() {
            return Err("SKU dan nama varian wajib diisi".to_string());
        }
        if input.purchase_price < 0 || input.selling_price < 0 || input.minimum_stock < 0 {
            return Err("Harga dan batas stok tidak boleh negatif".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        connection
            .execute(
                "UPDATE product_variants SET sku = ?1, name = ?2, purchase_price = ?3, selling_price = ?4, minimum_stock = ?5, updated_at = CURRENT_TIMESTAMP WHERE id = ?6 AND is_active = 1",
                rusqlite::params![sku, name, input.purchase_price, input.selling_price, input.minimum_stock, input.id],
            )
            .map_err(|error| {
                if error.to_string().contains("UNIQUE") {
                    "SKU sudah digunakan".to_string()
                } else {
                    format!("Unable to update variant: {error}")
                }
            })?;
        drop(connection);
        self.variants(None)?
            .into_iter()
            .find(|variant| variant.id == input.id)
            .ok_or_else(|| "Varian tidak ditemukan".to_string())
    }

    pub fn adjust_stock(&self, input: StockAdjustment) -> Result<VariantRow, String> {
        if input.quantity == 0 {
            return Err("Jumlah perubahan stok tidak boleh nol".to_string());
        }
        if input.reason.trim().is_empty() {
            return Err("Alasan penyesuaian wajib diisi".to_string());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Unable to start stock transaction: {error}"))?;
        let (stock_before, inventory_value_before, purchase_price, product_id): (i64, i64, i64, i64) = transaction
            .query_row(
                "SELECT current_stock, inventory_value, purchase_price, product_id FROM product_variants WHERE id = ?1 AND is_active = 1",
                [input.variant_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|_| "Varian tidak ditemukan".to_string())?;
        let stock_after = stock_before
            .checked_add(input.quantity)
            .ok_or_else(|| "Stok terlalu besar".to_string())?;
        if stock_after < 0 {
            return Err("Stok tidak mencukupi untuk perubahan ini".to_string());
        }
        let inventory_value_after = if input.quantity > 0 {
            inventory_value_before
                .checked_add(
                    input
                        .quantity
                        .checked_mul(purchase_price)
                        .ok_or_else(|| "Nilai inventori terlalu besar".to_string())?,
                )
                .ok_or_else(|| "Nilai inventori terlalu besar".to_string())?
        } else if stock_after == 0 {
            0
        } else {
            inventory_value_before
                .checked_mul(stock_after)
                .ok_or_else(|| "Nilai inventori terlalu besar".to_string())?
                / stock_before
        };
        let average_cost = if stock_after > 0 {
            inventory_value_after / stock_after
        } else {
            0
        };
        transaction.execute("UPDATE product_variants SET current_stock = ?1, inventory_value = ?2, purchase_price = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4", rusqlite::params![stock_after, inventory_value_after, average_cost, input.variant_id]).map_err(|error| format!("Unable to update stock: {error}"))?;
        transaction.execute("INSERT INTO inventory_movements (variant_id, movement_type, quantity, stock_before, stock_after, reason, moved_at) VALUES (?1, 'adjustment', ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)", rusqlite::params![input.variant_id, input.quantity, stock_before, stock_after, input.reason.trim()]).map_err(|error| format!("Unable to record movement: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Unable to commit stock transaction: {error}"))?;
        self.variants(Some(product_id))?
            .into_iter()
            .find(|variant| variant.id == input.variant_id)
            .ok_or_else(|| "Varian tidak ditemukan setelah pembaruan".to_string())
    }

    pub fn create_purchase(&self, input: NewPurchase) -> Result<TransactionResult, String> {
        if input.items.is_empty() {
            return Err("Pembelian harus memiliki minimal satu item".to_string());
        }
        if input.purchased_at.trim().is_empty() {
            return Err("Tanggal pembelian wajib diisi".to_string());
        }
        for item in &input.items {
            if item.quantity <= 0 || item.unit_cost < 0 {
                return Err(
                    "Jumlah harus lebih dari nol dan harga beli tidak boleh negatif".to_string(),
                );
            }
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Unable to start purchase transaction: {error}"))?;
        transaction
            .execute(
                "INSERT INTO purchases (reference, purchased_at, notes) VALUES (?1, ?2, ?3)",
                rusqlite::params![input.reference, input.purchased_at.trim(), input.notes],
            )
            .map_err(|error| format!("Unable to create purchase: {error}"))?;
        let purchase_id = transaction.last_insert_rowid();
        let mut total = 0_i64;
        for item in input.items {
            let (stock_before, inventory_value_before): (i64, i64) = transaction
                .query_row(
                    "SELECT current_stock, inventory_value FROM product_variants WHERE id = ?1 AND is_active = 1",
                    [item.variant_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|_| "Varian pembelian tidak ditemukan".to_string())?;
            let subtotal = item
                .quantity
                .checked_mul(item.unit_cost)
                .ok_or_else(|| "Nilai subtotal terlalu besar".to_string())?;
            total = total
                .checked_add(subtotal)
                .ok_or_else(|| "Total pembelian terlalu besar".to_string())?;
            let inventory_value_after = inventory_value_before
                .checked_add(subtotal)
                .ok_or_else(|| "Nilai inventori terlalu besar".to_string())?;
            let stock_after = stock_before
                .checked_add(item.quantity)
                .ok_or_else(|| "Stok terlalu besar".to_string())?;
            let average_cost = inventory_value_after / stock_after;
            transaction.execute("INSERT INTO purchase_items (purchase_id, variant_id, quantity, unit_cost, subtotal) VALUES (?1, ?2, ?3, ?4, ?5)", rusqlite::params![purchase_id, item.variant_id, item.quantity, item.unit_cost, subtotal]).map_err(|error| format!("Unable to create purchase item: {error}"))?;
            transaction.execute("UPDATE product_variants SET current_stock = ?1, inventory_value = ?2, purchase_price = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4", rusqlite::params![stock_after, inventory_value_after, average_cost, item.variant_id]).map_err(|error| format!("Unable to update purchased stock: {error}"))?;
            transaction.execute("INSERT INTO inventory_movements (variant_id, movement_type, quantity, stock_before, stock_after, reference, moved_at) VALUES (?1, 'purchase', ?2, ?3, ?4, ?5, ?6)", rusqlite::params![item.variant_id, item.quantity, stock_before, stock_after, purchase_id.to_string(), input.purchased_at.trim()]).map_err(|error| format!("Unable to record purchase movement: {error}"))?;
        }
        transaction
            .execute(
                "UPDATE purchases SET total_amount = ?1 WHERE id = ?2",
                rusqlite::params![total, purchase_id],
            )
            .map_err(|error| format!("Unable to update purchase total: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Unable to commit purchase: {error}"))?;
        Ok(TransactionResult {
            id: purchase_id,
            total_amount: total,
            total_profit: 0,
        })
    }

    pub fn create_sale(&self, input: NewSale) -> Result<TransactionResult, String> {
        if input.items.is_empty() {
            return Err("Penjualan harus memiliki minimal satu item".to_string());
        }
        if input.sold_at.trim().is_empty() {
            return Err("Tanggal penjualan wajib diisi".to_string());
        }
        for item in &input.items {
            if item.quantity <= 0 || item.unit_price.is_some_and(|price| price < 0) {
                return Err(
                    "Jumlah harus lebih dari nol dan harga jual tidak boleh negatif".to_string(),
                );
            }
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Unable to start sale transaction: {error}"))?;
        transaction
            .execute(
                "INSERT INTO sales (reference, sold_at, notes) VALUES (?1, ?2, ?3)",
                rusqlite::params![input.reference, input.sold_at.trim(), input.notes],
            )
            .map_err(|error| format!("Unable to create sale: {error}"))?;
        let sale_id = transaction.last_insert_rowid();
        let mut total = 0_i64;
        let mut total_profit = 0_i64;
        for item in input.items {
            let (stock_before, inventory_value_before, selling_price): (i64, i64, i64) = transaction.query_row("SELECT current_stock, inventory_value, selling_price FROM product_variants WHERE id = ?1 AND is_active = 1", [item.variant_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).map_err(|_| "Varian penjualan tidak ditemukan atau tidak aktif".to_string())?;
            if stock_before < item.quantity {
                return Err(format!(
                    "Stok tidak mencukupi untuk varian {}",
                    item.variant_id
                ));
            }
            let unit_price = item.unit_price.unwrap_or(selling_price);
            let subtotal = item
                .quantity
                .checked_mul(unit_price)
                .ok_or_else(|| "Nilai subtotal terlalu besar".to_string())?;
            let cogs = if item.quantity == stock_before {
                inventory_value_before
            } else {
                inventory_value_before
                    .checked_mul(item.quantity)
                    .ok_or_else(|| "Nilai HPP terlalu besar".to_string())?
                    / stock_before
            };
            let inventory_value_after = inventory_value_before - cogs;
            let stock_after = stock_before - item.quantity;
            let unit_cost = cogs / item.quantity;
            let profit = item
                .quantity
                .checked_mul(unit_price)
                .and_then(|revenue| revenue.checked_sub(cogs))
                .ok_or_else(|| "Nilai keuntungan terlalu besar".to_string())?;
            total = total
                .checked_add(subtotal)
                .ok_or_else(|| "Total penjualan terlalu besar".to_string())?;
            total_profit = total_profit
                .checked_add(profit)
                .ok_or_else(|| "Total keuntungan terlalu besar".to_string())?;
            transaction.execute("INSERT INTO sale_items (sale_id, variant_id, quantity, unit_price, unit_cost, subtotal, profit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", rusqlite::params![sale_id, item.variant_id, item.quantity, unit_price, unit_cost, subtotal, profit]).map_err(|error| format!("Unable to create sale item: {error}"))?;
            transaction.execute("UPDATE product_variants SET current_stock = ?1, inventory_value = ?2, purchase_price = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4", rusqlite::params![stock_after, inventory_value_after, if stock_after > 0 { inventory_value_after / stock_after } else { 0 }, item.variant_id]).map_err(|error| format!("Unable to update sold stock: {error}"))?;
            transaction.execute("INSERT INTO inventory_movements (variant_id, movement_type, quantity, stock_before, stock_after, reference, moved_at) VALUES (?1, 'sale', ?2, ?3, ?4, ?5, ?6)", rusqlite::params![item.variant_id, -item.quantity, stock_before, stock_after, sale_id.to_string(), input.sold_at.trim()]).map_err(|error| format!("Unable to record movement: {error}"))?;
        }
        transaction
            .execute(
                "UPDATE sales SET total_amount = ?1, total_profit = ?2 WHERE id = ?3",
                rusqlite::params![total, total_profit, sale_id],
            )
            .map_err(|error| format!("Unable to update sale totals: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Unable to commit sale: {error}"))?;
        Ok(TransactionResult {
            id: sale_id,
            total_amount: total,
            total_profit,
        })
    }

    pub fn evaluate_smart(
        &self,
        request: SmartEvaluationRequest,
    ) -> Result<SmartEvaluationResponse, String> {
        if request.period_start.trim().is_empty() || request.period_end.trim().is_empty() {
            return Err("Periode evaluasi wajib diisi".to_string());
        }
        if request.period_start > request.period_end {
            return Err("Tanggal mulai tidak boleh setelah tanggal akhir".to_string());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT v.id, v.name, v.sku, v.current_stock, v.purchase_price, COALESCE(SUM(CASE WHEN s.id IS NOT NULL THEN si.quantity ELSE 0 END), 0), COALESCE(SUM(CASE WHEN s.id IS NOT NULL THEN si.profit ELSE 0 END), 0) FROM product_variants v LEFT JOIN sale_items si ON si.variant_id = v.id LEFT JOIN sales s ON s.id = si.sale_id AND s.sold_at >= ?1 AND s.sold_at <= ?2 WHERE v.is_active = 1 GROUP BY v.id ORDER BY v.id").map_err(|error| format!("Unable to prepare SMART input: {error}"))?;
        let rows = statement
            .query_map(
                rusqlite::params![request.period_start.trim(), request.period_end.trim()],
                |row| {
                    Ok(crate::smart::SmartInput {
                        variant_id: row.get(0)?,
                        variant_name: row.get(1)?,
                        sku: row.get(2)?,
                        stock: row.get(3)?,
                        purchase_price: row.get(4)?,
                        units_sold: row.get(5)?,
                        profit: row.get(6)?,
                    })
                },
            )
            .map_err(|error| format!("Unable to read SMART input: {error}"))?;
        let inputs = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map SMART input: {error}"))?;
        drop(statement);
        let results = crate::smart::calculate(inputs);
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Unable to start SMART transaction: {error}"))?;
        transaction.execute("INSERT INTO smart_evaluations (period_start, period_end, costing_method, total_weight) VALUES (?1, ?2, ?3, 1.0)", rusqlite::params![request.period_start.trim(), request.period_end.trim(), request.costing_method.trim()]).map_err(|error| format!("Unable to create SMART evaluation: {error}"))?;
        let evaluation_id = transaction.last_insert_rowid();
        let mut criterion_ids = std::collections::HashMap::new();
        let mut criteria_statement = transaction
            .prepare("SELECT id, code FROM smart_criteria WHERE is_active = 1")
            .map_err(|error| format!("Unable to read SMART criteria: {error}"))?;
        let criteria = criteria_statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, i64>(0)?))
            })
            .map_err(|error| format!("Unable to map SMART criteria: {error}"))?;
        for criterion in criteria {
            let (code, id) =
                criterion.map_err(|error| format!("Unable to read SMART criterion: {error}"))?;
            criterion_ids.insert(code, id);
        }
        drop(criteria_statement);
        for result in &results {
            for criterion in &result.criteria {
                let criterion_id = criterion_ids
                    .get(&criterion.code)
                    .ok_or_else(|| format!("Missing SMART criterion {}", criterion.code))?;
                transaction.execute("INSERT INTO smart_evaluation_details (evaluation_id, variant_id, criterion_id, raw_value, utility_value, weighted_contribution) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", rusqlite::params![evaluation_id, result.variant_id, criterion_id, criterion.raw_value, criterion.utility_value, criterion.weighted_contribution]).map_err(|error| format!("Unable to store SMART detail: {error}"))?;
            }
            transaction.execute("INSERT INTO smart_ranking_results (evaluation_id, variant_id, final_score, ranking_position) VALUES (?1, ?2, ?3, ?4)", rusqlite::params![evaluation_id, result.variant_id, result.score, result.ranking_position as i64]).map_err(|error| format!("Unable to store SMART ranking: {error}"))?;
        }
        transaction
            .commit()
            .map_err(|error| format!("Unable to commit SMART evaluation: {error}"))?;
        Ok(SmartEvaluationResponse {
            evaluation_id,
            period_start: request.period_start,
            period_end: request.period_end,
            results,
        })
    }

    pub fn report_summary(&self) -> Result<ReportSummary, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let scalar = |sql: &str| -> Result<i64, String> {
            connection
                .query_row(sql, [], |row| row.get(0))
                .map_err(|error| format!("Unable to calculate report: {error}"))
        };
        Ok(ReportSummary {
            active_products: scalar("SELECT COUNT(*) FROM products WHERE is_active = 1")?,
            variant_count: scalar("SELECT COUNT(*) FROM product_variants WHERE is_active = 1")?,
            total_stock: scalar(
                "SELECT COALESCE(SUM(current_stock), 0) FROM product_variants WHERE is_active = 1",
            )?,
            sales_today: scalar(
                "SELECT COALESCE(SUM(si.quantity), 0) FROM sale_items si JOIN sales s ON s.id = si.sale_id WHERE date(s.sold_at) = date('now', 'localtime')",
            )?,
            revenue_today: scalar(
                "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE date(sold_at) = date('now', 'localtime')",
            )?,
            profit_today: scalar(
                "SELECT COALESCE(SUM(total_profit), 0) FROM sales WHERE date(sold_at) = date('now', 'localtime')",
            )?,
            low_stock_count: scalar(
                "SELECT COUNT(*) FROM product_variants WHERE is_active = 1 AND current_stock <= minimum_stock",
            )?,
        })
    }

    pub fn report_summary_range(
        &self,
        request: ReportRangeRequest,
    ) -> Result<ReportSummary, String> {
        if request.period_start.trim().is_empty() || request.period_end.trim().is_empty() {
            return Err("Periode laporan wajib diisi".to_string());
        }
        if request.period_start > request.period_end {
            return Err("Tanggal mulai laporan tidak boleh setelah tanggal akhir".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let range_scalar = |sql: &str| -> Result<i64, String> {
            connection
                .query_row(
                    sql,
                    rusqlite::params![request.period_start.trim(), request.period_end.trim()],
                    |row| row.get(0),
                )
                .map_err(|error| format!("Unable to calculate report: {error}"))
        };
        let scalar = |sql: &str| -> Result<i64, String> {
            connection
                .query_row(sql, [], |row| row.get(0))
                .map_err(|error| format!("Unable to calculate report: {error}"))
        };
        Ok(ReportSummary {
            active_products: scalar("SELECT COUNT(*) FROM products WHERE is_active = 1")?,
            variant_count: scalar("SELECT COUNT(*) FROM product_variants WHERE is_active = 1")?,
            total_stock: scalar(
                "SELECT COALESCE(SUM(current_stock), 0) FROM product_variants WHERE is_active = 1",
            )?,
            sales_today: range_scalar(
                "SELECT COALESCE(SUM(si.quantity), 0) FROM sale_items si JOIN sales s ON s.id = si.sale_id WHERE date(s.sold_at) BETWEEN date(?1) AND date(?2)",
            )?,
            revenue_today: range_scalar(
                "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE date(sold_at) BETWEEN date(?1) AND date(?2)",
            )?,
            profit_today: range_scalar(
                "SELECT COALESCE(SUM(total_profit), 0) FROM sales WHERE date(sold_at) BETWEEN date(?1) AND date(?2)",
            )?,
            low_stock_count: scalar(
                "SELECT COUNT(*) FROM product_variants WHERE is_active = 1 AND current_stock <= minimum_stock",
            )?,
        })
    }

    pub fn variant_report_stats(
        &self,
        request: VariantReportRequest,
    ) -> Result<VariantReportStats, String> {
        if request.variant_id <= 0 || request.period_start.trim().is_empty() || request.period_end.trim().is_empty() {
            return Err("Varian dan periode laporan wajib diisi".to_string());
        }
        if request.period_start > request.period_end {
            return Err("Tanggal mulai laporan tidak boleh setelah tanggal akhir".to_string());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let (product_name, variant_name, current_stock): (String, String, i64) = connection
            .query_row(
                "SELECT p.name, v.name, v.current_stock FROM product_variants v JOIN products p ON p.id = v.product_id WHERE v.id = ?1 AND v.is_active = 1 AND p.is_active = 1",
                [request.variant_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| "Varian laporan tidak ditemukan".to_string())?;
        let (units_sold, revenue, profit): (i64, i64, i64) = connection
            .query_row(
                "SELECT COALESCE(SUM(si.quantity), 0), COALESCE(SUM(si.subtotal), 0), COALESCE(SUM(si.profit), 0) FROM sale_items si JOIN sales s ON s.id = si.sale_id WHERE si.variant_id = ?1 AND date(s.sold_at) BETWEEN date(?2) AND date(?3)",
                rusqlite::params![request.variant_id, request.period_start.trim(), request.period_end.trim()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|error| format!("Unable to calculate variant report: {error}"))?;
        let mut statement = connection
            .prepare("SELECT date(s.sold_at), COALESCE(SUM(si.quantity), 0), COALESCE(SUM(si.subtotal), 0), COALESCE(SUM(si.profit), 0) FROM sale_items si JOIN sales s ON s.id = si.sale_id WHERE si.variant_id = ?1 AND date(s.sold_at) BETWEEN date(?2) AND date(?3) GROUP BY date(s.sold_at) ORDER BY date(s.sold_at)")
            .map_err(|error| format!("Unable to prepare variant report: {error}"))?;
        let points = statement
            .query_map(rusqlite::params![request.variant_id, request.period_start.trim(), request.period_end.trim()], |row| {
                Ok(ReportPoint { date: row.get(0)?, units_sold: row.get(1)?, revenue: row.get(2)?, profit: row.get(3)? })
            })
            .map_err(|error| format!("Unable to read variant report: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map variant report: {error}"))?;
        Ok(VariantReportStats { variant_id: request.variant_id, product_name, variant_name, current_stock, units_sold, revenue, profit, points })
    }

    pub fn smart_history(&self) -> Result<Vec<SmartHistoryRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT e.id, e.period_start, e.period_end, e.evaluated_at, e.costing_method, COUNT(r.id) FROM smart_evaluations e LEFT JOIN smart_ranking_results r ON r.evaluation_id = e.id GROUP BY e.id ORDER BY e.evaluated_at DESC").map_err(|error| format!("Unable to prepare SMART history: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(SmartHistoryRow {
                    id: row.get(0)?,
                    period_start: row.get(1)?,
                    period_end: row.get(2)?,
                    evaluated_at: row.get(3)?,
                    costing_method: row.get(4)?,
                    ranking_count: row.get(5)?,
                })
            })
            .map_err(|error| format!("Unable to read SMART history: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map SMART history: {error}"))
    }

    pub fn movements(&self) -> Result<Vec<MovementRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT m.id, p.name, v.name, m.movement_type, m.quantity, m.stock_after, m.moved_at, m.reason, m.reference FROM inventory_movements m JOIN product_variants v ON v.id = m.variant_id JOIN products p ON p.id = v.product_id ORDER BY m.moved_at DESC, m.id DESC LIMIT 100").map_err(|error| format!("Unable to prepare movements: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(MovementRow {
                    id: row.get(0)?,
                    product_name: row.get(1)?,
                    variant_name: row.get(2)?,
                    movement_type: row.get(3)?,
                    quantity: row.get(4)?,
                    stock_after: row.get(5)?,
                    moved_at: row.get(6)?,
                    reason: row.get(7)?,
                    reference: row.get(8)?,
                })
            })
            .map_err(|error| format!("Unable to read movements: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map movements: {error}"))
    }

    pub fn sales_history(&self) -> Result<Vec<SaleRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT id, reference, sold_at, total_amount, total_profit FROM sales ORDER BY sold_at DESC, id DESC LIMIT 100").map_err(|error| format!("Unable to prepare sales history: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(SaleRow {
                    id: row.get(0)?,
                    reference: row.get(1)?,
                    sold_at: row.get(2)?,
                    total_amount: row.get(3)?,
                    total_profit: row.get(4)?,
                })
            })
            .map_err(|error| format!("Unable to read sales history: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map sales history: {error}"))
    }

    pub fn purchases_history(&self) -> Result<Vec<PurchaseRow>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT id, reference, purchased_at, total_amount FROM purchases ORDER BY purchased_at DESC, id DESC LIMIT 100").map_err(|error| format!("Unable to prepare purchases history: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(PurchaseRow {
                    id: row.get(0)?,
                    reference: row.get(1)?,
                    purchased_at: row.get(2)?,
                    total_amount: row.get(3)?,
                })
            })
            .map_err(|error| format!("Unable to read purchases history: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Unable to map purchases history: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Database, NewCategory, NewProduct, NewPurchase, NewSale, NewVariant, PurchaseItemInput,
        ReportRangeRequest, SaleItemInput,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn purchase_and_sale_preserve_stock_and_profit_invariants() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("siambruk-integration-{suffix}"));
        let database = Database::open(&directory).expect("database should initialize");
        let category = database
            .create_category(NewCategory {
                name: "Snacks".to_string(),
                description: None,
            })
            .expect("category should be created");
        let product = database
            .create_product(NewProduct {
                name: "Keripik".to_string(),
                category_id: Some(category.id),
                description: None,
            })
            .expect("product should be created");
        let variant = database
            .create_variant(NewVariant {
                product_id: product.id,
                sku: "KRP-250".to_string(),
                name: "250g".to_string(),
                purchase_price: 100,
                selling_price: 150,
                minimum_stock: 1,
            })
            .expect("variant should be created");
        database
            .create_purchase(NewPurchase {
                reference: Some("PO-1".to_string()),
                purchased_at: "2026-11-26".to_string(),
                notes: None,
                items: vec![PurchaseItemInput {
                    variant_id: variant.id,
                    quantity: 5,
                    unit_cost: 100,
                }],
            })
            .expect("purchase should be recorded");
        assert_eq!(database.status().unwrap().variant_count, 1);
        assert_eq!(database.variants(None).unwrap()[0].current_stock, 5);
        let sale = database
            .create_sale(NewSale {
                reference: Some("SO-1".to_string()),
                sold_at: "2026-11-26".to_string(),
                notes: None,
                items: vec![SaleItemInput {
                    variant_id: variant.id,
                    quantity: 2,
                    unit_price: Some(150),
                }],
            })
            .expect("sale should be recorded");
        assert_eq!(sale.total_amount, 300);
        assert_eq!(sale.total_profit, 100);
        assert_eq!(database.variants(None).unwrap()[0].current_stock, 3);
        assert_eq!(database.purchases_history().unwrap().len(), 1);
        assert_eq!(database.sales_history().unwrap().len(), 1);
        let report = database
            .report_summary_range(ReportRangeRequest {
                period_start: "2026-11-26".to_string(),
                period_end: "2026-11-26".to_string(),
            })
            .unwrap();
        assert_eq!(report.sales_today, 2);
        assert_eq!(report.revenue_today, 300);
        assert_eq!(report.profit_today, 100);
        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn repeated_purchases_use_moving_average_and_rollback_invalid_purchase() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("siambruk-average-{suffix}"));
        let database = Database::open(&directory).expect("database should initialize");
        let product = database
            .create_product(NewProduct {
                name: "Snack".to_string(),
                category_id: None,
                description: None,
            })
            .unwrap();
        let variant = database
            .create_variant(NewVariant {
                product_id: product.id,
                sku: "SNK-1".to_string(),
                name: "250gr".to_string(),
                purchase_price: 100,
                selling_price: 250,
                minimum_stock: 0,
            })
            .unwrap();
        database
            .create_purchase(NewPurchase {
                reference: None,
                purchased_at: "2026-11-26".to_string(),
                notes: None,
                items: vec![PurchaseItemInput {
                    variant_id: variant.id,
                    quantity: 10,
                    unit_cost: 100,
                }],
            })
            .unwrap();
        database
            .create_purchase(NewPurchase {
                reference: None,
                purchased_at: "2026-11-26".to_string(),
                notes: None,
                items: vec![PurchaseItemInput {
                    variant_id: variant.id,
                    quantity: 10,
                    unit_cost: 200,
                }],
            })
            .unwrap();
        let after_purchases = database.variants(None).unwrap().remove(0);
        assert_eq!(after_purchases.current_stock, 20);
        assert_eq!(after_purchases.purchase_price, 150);
        let sale = database
            .create_sale(NewSale {
                reference: None,
                sold_at: "2026-11-26".to_string(),
                notes: None,
                items: vec![SaleItemInput {
                    variant_id: variant.id,
                    quantity: 4,
                    unit_price: Some(250),
                }],
            })
            .unwrap();
        assert_eq!(sale.total_profit, 400);
        assert_eq!(database.variants(None).unwrap().remove(0).current_stock, 16);
        let invalid = database.create_purchase(NewPurchase {
            reference: None,
            purchased_at: "2026-11-26".to_string(),
            notes: None,
            items: vec![
                PurchaseItemInput {
                    variant_id: variant.id,
                    quantity: 1,
                    unit_cost: 300,
                },
                PurchaseItemInput {
                    variant_id: 999_999,
                    quantity: 1,
                    unit_cost: 300,
                },
            ],
        });
        assert!(invalid.is_err());
        assert_eq!(database.variants(None).unwrap().remove(0).current_stock, 16);
        std::fs::remove_dir_all(directory).ok();
    }
}
