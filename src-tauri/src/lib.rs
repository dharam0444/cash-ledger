mod commands;
mod db;
mod errors;
mod repositories;
mod security;
mod services;

use db::Database;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = Database::open(app.handle())?;
            database.initialize()?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::initialize_app,
            commands::auth_login,
            commands::get_settings,
            commands::update_settings,
            commands::list_banks,
            commands::list_customers,
            commands::search_customer,
            commands::get_customer,
            commands::create_customer,
            commands::update_customer,
            commands::create_transaction,
            commands::get_daily_summary,
            commands::get_daily_transactions,
            commands::get_daily_report,
            commands::get_license_status,
            commands::complete_client_setup,
            commands::reset_admin_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cash Ledger");
}
