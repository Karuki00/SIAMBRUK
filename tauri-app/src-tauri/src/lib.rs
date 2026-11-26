mod database;
mod smart;

use database::{
    CategoryRow, Database, DatabaseStatus, MovementRow, NewCategory, NewProduct, NewPurchase,
    NewSale, NewVariant, ProductRow, PurchaseRow, ReportRangeRequest, ReportSummary, SaleRow,
    SmartEvaluationRequest, SmartEvaluationResponse, SmartHistoryRow, StockAdjustment,
    TransactionResult, UpdateCategory, UpdateProduct, VariantRow,
};
use tauri::{Manager, State};

#[tauri::command]
fn database_status(database: State<'_, Database>) -> Result<DatabaseStatus, String> {
    database.status()
}

#[tauri::command]
fn list_categories(database: State<'_, Database>) -> Result<Vec<CategoryRow>, String> {
    database.categories()
}

#[tauri::command]
fn create_category(
    database: State<'_, Database>,
    input: NewCategory,
) -> Result<CategoryRow, String> {
    database.create_category(input)
}

#[tauri::command]
fn delete_category(database: State<'_, Database>, category_id: i64) -> Result<(), String> {
    database.delete_category(category_id)
}

#[tauri::command]
fn update_category(database: State<'_, Database>, input: UpdateCategory) -> Result<(), String> {
    database.update_category(input)
}

#[tauri::command]
fn list_products(database: State<'_, Database>) -> Result<Vec<ProductRow>, String> {
    database.products()
}

#[tauri::command]
fn create_product(database: State<'_, Database>, input: NewProduct) -> Result<ProductRow, String> {
    database.create_product(input)
}

#[tauri::command]
fn delete_product(database: State<'_, Database>, product_id: i64) -> Result<(), String> {
    database.delete_product(product_id)
}

#[tauri::command]
fn update_product(database: State<'_, Database>, input: UpdateProduct) -> Result<(), String> {
    database.update_product(input)
}

#[tauri::command]
fn list_variants(
    database: State<'_, Database>,
    product_id: Option<i64>,
) -> Result<Vec<VariantRow>, String> {
    database.variants(product_id)
}

#[tauri::command]
fn create_variant(database: State<'_, Database>, input: NewVariant) -> Result<VariantRow, String> {
    database.create_variant(input)
}

#[tauri::command]
fn adjust_stock(
    database: State<'_, Database>,
    input: StockAdjustment,
) -> Result<VariantRow, String> {
    database.adjust_stock(input)
}

#[tauri::command]
fn create_purchase(
    database: State<'_, Database>,
    input: NewPurchase,
) -> Result<TransactionResult, String> {
    database.create_purchase(input)
}

#[tauri::command]
fn create_sale(database: State<'_, Database>, input: NewSale) -> Result<TransactionResult, String> {
    database.create_sale(input)
}

#[tauri::command]
fn calculate_smart(input: Vec<smart::SmartInput>) -> Vec<smart::SmartResult> {
    smart::calculate(input)
}

#[tauri::command]
fn evaluate_smart(
    database: State<'_, Database>,
    request: SmartEvaluationRequest,
) -> Result<SmartEvaluationResponse, String> {
    database.evaluate_smart(request)
}

#[tauri::command]
fn report_summary(database: State<'_, Database>) -> Result<ReportSummary, String> {
    database.report_summary()
}

#[tauri::command]
fn report_summary_range(
    database: State<'_, Database>,
    request: ReportRangeRequest,
) -> Result<ReportSummary, String> {
    database.report_summary_range(request)
}

#[tauri::command]
fn list_smart_history(database: State<'_, Database>) -> Result<Vec<SmartHistoryRow>, String> {
    database.smart_history()
}

#[tauri::command]
fn list_movements(database: State<'_, Database>) -> Result<Vec<MovementRow>, String> {
    database.movements()
}

#[tauri::command]
fn list_sales_history(database: State<'_, Database>) -> Result<Vec<SaleRow>, String> {
    database.sales_history()
}

#[tauri::command]
fn list_purchases_history(database: State<'_, Database>) -> Result<Vec<PurchaseRow>, String> {
    database.purchases_history()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    builder
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            let database = Database::open(&app_data_dir).map_err(std::io::Error::other)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            database_status,
            list_categories,
            create_category,
            delete_category,
            update_category,
            list_products,
            create_product,
            delete_product,
            update_product,
            list_variants,
            create_variant,
            adjust_stock,
            create_purchase,
            create_sale,
            calculate_smart,
            evaluate_smart,
            report_summary,
            report_summary_range,
            list_smart_history,
            list_movements,
            list_sales_history,
            list_purchases_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
