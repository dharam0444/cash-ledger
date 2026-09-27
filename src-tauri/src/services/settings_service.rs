use crate::errors::{AppError, AppResult};
use crate::repositories::settings_repository;
use rusqlite::Connection;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub shop_name: String,
    pub owner_name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub automatic_backup_enabled: bool,
    pub backup_retention_days: i64,
    pub auto_lock_minutes: i64,
}

pub fn get_settings(connection: &Connection) -> AppResult<AppSettings> {
    Ok(AppSettings {
        shop_name: get_string(connection, "shop_name", "Cash Ledger")?,
        owner_name: settings_repository::get_value(connection, "owner_name")?,
        phone: settings_repository::get_value(connection, "phone")?,
        address: settings_repository::get_value(connection, "address")?,
        automatic_backup_enabled: get_bool(connection, "automatic_backup_enabled", true)?,
        backup_retention_days: get_i64(connection, "backup_retention_days", 30)?,
        auto_lock_minutes: get_i64(connection, "auto_lock_minutes", 10)?,
    })
}

pub fn update_settings(connection: &Connection, settings: AppSettings) -> AppResult<AppSettings> {
    if settings.shop_name.trim().is_empty() {
        return Err(AppError::Validation("Shop name is required.".to_string()));
    }

    settings_repository::set_value(connection, "shop_name", settings.shop_name.trim())?;
    set_optional(connection, "owner_name", settings.owner_name.as_deref())?;
    set_optional(connection, "phone", settings.phone.as_deref())?;
    set_optional(connection, "address", settings.address.as_deref())?;
    settings_repository::set_value(
        connection,
        "automatic_backup_enabled",
        if settings.automatic_backup_enabled {
            "true"
        } else {
            "false"
        },
    )?;
    settings_repository::set_value(
        connection,
        "backup_retention_days",
        &settings.backup_retention_days.to_string(),
    )?;
    settings_repository::set_value(
        connection,
        "auto_lock_minutes",
        &settings.auto_lock_minutes.to_string(),
    )?;

    get_settings(connection)
}

fn get_string(connection: &Connection, key: &str, fallback: &str) -> AppResult<String> {
    Ok(settings_repository::get_value(connection, key)?.unwrap_or_else(|| fallback.to_string()))
}

fn get_i64(connection: &Connection, key: &str, fallback: i64) -> AppResult<i64> {
    Ok(settings_repository::get_value(connection, key)?
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(fallback))
}

fn get_bool(connection: &Connection, key: &str, fallback: bool) -> AppResult<bool> {
    Ok(settings_repository::get_value(connection, key)?
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(fallback))
}

fn set_optional(connection: &Connection, key: &str, value: Option<&str>) -> AppResult<()> {
    settings_repository::set_value(connection, key, value.unwrap_or("").trim())
}
