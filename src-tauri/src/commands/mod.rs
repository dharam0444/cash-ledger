use crate::db::{Database, DbStatus};
use crate::errors::AppResult;
use crate::services::{auth_service, customer_service, settings_service, transaction_service};
use tauri::State;

#[tauri::command]
pub fn initialize_app(database: State<'_, Database>) -> AppResult<DbStatus> {
    {
        let connection = database.lock()?;
        auth_service::ensure_default_admin(&connection)?;
    }
    database.status()
}

#[tauri::command]
pub fn auth_login(
    database: State<'_, Database>,
    input: auth_service::LoginInput,
) -> AppResult<auth_service::AuthenticatedUser> {
    let connection = database.lock()?;
    auth_service::login(&connection, input)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetInput {
    pub machine_code: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[tauri::command]
pub fn reset_admin_password(
    database: State<'_, Database>,
    input: PasswordResetInput,
) -> AppResult<()> {
    let connection = database.lock()?;
    auth_service::reset_admin_password(&connection, &input.machine_code, &input.recovery_code, &input.new_password)
}

#[tauri::command]
pub fn get_settings(database: State<'_, Database>) -> AppResult<settings_service::AppSettings> {
    let connection = database.lock()?;
    settings_service::get_settings(&connection)
}

#[tauri::command]
pub fn update_settings(
    database: State<'_, Database>,
    settings: settings_service::AppSettings,
) -> AppResult<settings_service::AppSettings> {
    let connection = database.lock()?;
    settings_service::update_settings(&connection, settings)
}

#[tauri::command]
pub fn list_banks(database: State<'_, Database>) -> AppResult<Vec<customer_service::BankOption>> {
    let connection = database.lock()?;
    customer_service::list_banks(&connection)
}

#[tauri::command]
pub fn list_customers(
    database: State<'_, Database>,
) -> AppResult<Vec<customer_service::CustomerView>> {
    let connection = database.lock()?;
    customer_service::list_customers(&connection)
}

#[tauri::command]
pub fn search_customer(
    database: State<'_, Database>,
    input: customer_service::SearchCustomerInput,
) -> AppResult<Vec<customer_service::CustomerView>> {
    let connection = database.lock()?;
    customer_service::search_customer(&connection, input)
}

#[tauri::command]
pub fn get_customer(
    database: State<'_, Database>,
    customer_id: i64,
) -> AppResult<customer_service::CustomerView> {
    let connection = database.lock()?;
    customer_service::get_customer(&connection, customer_id)
}

#[tauri::command]
pub fn create_customer(
    database: State<'_, Database>,
    input: customer_service::CustomerInput,
    user_id: i64,
) -> AppResult<customer_service::CustomerView> {
    let mut connection = database.lock()?;
    customer_service::create_customer(&mut connection, input, user_id)
}

#[tauri::command]
pub fn update_customer(
    database: State<'_, Database>,
    input: customer_service::UpdateCustomerInput,
    user_id: i64,
) -> AppResult<customer_service::CustomerView> {
    let mut connection = database.lock()?;
    customer_service::update_customer(&mut connection, input, user_id)
}

#[tauri::command]
pub fn create_transaction(
    database: State<'_, Database>,
    input: transaction_service::CreateTransactionInput,
) -> AppResult<transaction_service::TransactionView> {
    let mut connection = database.lock()?;
    transaction_service::create_transaction(&mut connection, input)
}

#[tauri::command]
pub fn get_daily_summary(
    database: State<'_, Database>,
    input: transaction_service::DailyQueryInput,
) -> AppResult<transaction_service::DailySummary> {
    let connection = database.lock()?;
    transaction_service::get_daily_summary(&connection, input)
}

#[tauri::command]
pub fn get_daily_transactions(
    database: State<'_, Database>,
    input: transaction_service::DailyQueryInput,
) -> AppResult<Vec<transaction_service::TransactionView>> {
    let connection = database.lock()?;
    transaction_service::get_daily_transactions(&connection, input)
}

#[tauri::command]
pub fn get_daily_report(
    database: State<'_, Database>,
    input: transaction_service::DailyQueryInput,
) -> AppResult<transaction_service::DailyReport> {
    let connection = database.lock()?;
    transaction_service::get_daily_report(&connection, input)
}

#[tauri::command]
pub fn get_license_status(
    database: State<'_, Database>,
) -> AppResult<crate::services::license_service::LicenseStatus> {
    let connection = database.lock()?;
    crate::services::license_service::license_status(&connection)
}

#[tauri::command]
pub fn complete_client_setup(
    database: State<'_, Database>,
    input: crate::services::license_service::ClientSetupInput,
) -> AppResult<crate::services::license_service::LicenseStatus> {
    let connection = database.lock()?;
    crate::services::license_service::complete_client_setup(&connection, input)
}
