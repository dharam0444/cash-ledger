use crate::errors::{AppError, AppResult};
use crate::repositories::settings_repository;
use crate::security;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

const VENDOR_LICENSE_SECRET: &str = "CHANGE_THIS_VENDOR_SECRET_BEFORE_RELEASE_2026";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub machine_code: String,
    pub is_activated: bool,
    pub client_name: Option<String>,
    pub client_mobile: Option<String>,
    pub default_bank_id: Option<i64>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSetupInput {
    pub client_name: String,
    pub client_mobile: String,
    pub admin_password: String,
    pub license_key: String,
    pub default_bank_id: Option<i64>,
}

pub fn license_status(connection: &Connection) -> AppResult<LicenseStatus> {
    let machine_code = machine_code();
    let stored_license = settings_repository::get_value(connection, "license_key")?;
    let is_activated = stored_license
        .as_deref()
        .map(|key| validate_license_key(&machine_code, key))
        .unwrap_or(false);

    Ok(LicenseStatus {
        machine_code,
        is_activated,
        client_name: settings_repository::get_value(connection, "client_name")?,
        client_mobile: settings_repository::get_value(connection, "client_mobile")?,
        default_bank_id: settings_repository::get_value(connection, "default_bank_id")?
            .and_then(|value| value.parse::<i64>().ok()),
    })
}

pub fn complete_client_setup(
    connection: &Connection,
    input: ClientSetupInput,
) -> AppResult<LicenseStatus> {
    let client_name = input.client_name.trim();
    let client_mobile = normalize_mobile(&input.client_mobile)?;
    let admin_password = input.admin_password.trim();
    let license_key = input.license_key.trim().to_uppercase();
    let machine_code = machine_code();

    if client_name.len() < 2 {
        return Err(AppError::Validation("Client name is required.".to_string()));
    }
    if admin_password.len() < 6 {
        return Err(AppError::Validation(
            "Admin password must be at least 6 characters.".to_string(),
        ));
    }
    if !validate_license_key(&machine_code, &license_key) {
        return Err(AppError::Validation(
            "Invalid license key for this machine.".to_string(),
        ));
    }

    let (password_hash, password_salt) = security::hash_password(admin_password)?;
    connection.execute(
        "UPDATE users SET password_hash = ?1, password_salt = ?2 WHERE username = 'admin'",
        params![password_hash, password_salt],
    )?;

    settings_repository::set_value(connection, "client_name", client_name)?;
    settings_repository::set_value(connection, "client_mobile", &client_mobile)?;
    settings_repository::set_value(connection, "license_key", &license_key)?;
    settings_repository::set_value(connection, "license_activated", "true")?;
    if let Some(default_bank_id) = input.default_bank_id {
        if default_bank_id > 0 {
            settings_repository::set_value(
                connection,
                "default_bank_id",
                &default_bank_id.to_string(),
            )?;
        }
    }

    license_status(connection)
}

pub fn generate_license_for_machine(machine_code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(VENDOR_LICENSE_SECRET.as_bytes());
    hasher.update(b"|");
    hasher.update(machine_code.trim().to_uppercase().as_bytes());
    let hex = hex::encode(hasher.finalize()).to_uppercase();
    format!(
        "{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..16],
        &hex[16..24],
        &hex[24..32]
    )
}

fn validate_license_key(machine_code: &str, license_key: &str) -> bool {
    normalize_license(license_key) == normalize_license(&generate_license_for_machine(machine_code))
}

fn machine_code() -> String {
    let mut hasher = Sha256::new();
    for key in ["COMPUTERNAME", "HOSTNAME", "USERNAME", "USER"] {
        if let Ok(value) = std::env::var(key) {
            hasher.update(key.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
            hasher.update(b";");
        }
    }
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(b";");
    hasher.update(std::env::consts::ARCH.as_bytes());
    let hex = hex::encode(hasher.finalize()).to_uppercase();
    format!("{}-{}-{}", &hex[0..6], &hex[6..12], &hex[12..18])
}

fn normalize_license(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}

fn normalize_mobile(input: &str) -> AppResult<String> {
    let digits: String = input.chars().filter(|ch| ch.is_ascii_digit()).collect();
    let mobile = if digits.len() == 12 && digits.starts_with("91") {
        digits[2..].to_string()
    } else {
        digits
    };
    if mobile.len() == 10 && matches!(mobile.as_bytes()[0], b'6'..=b'9') {
        Ok(mobile)
    } else {
        Err(AppError::Validation(
            "Enter a valid 10 digit mobile number.".to_string(),
        ))
    }
}
