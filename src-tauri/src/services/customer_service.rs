use crate::errors::{AppError, AppResult};
use crate::repositories::settings_repository;
use crate::security;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankOption {
    pub id: i64,
    pub name: String,
    pub short_name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerAccountView {
    pub id: i64,
    pub bank_id: i64,
    pub bank_name: String,
    pub account_last4: String,
    pub account_masked: String,
    pub account_display: String,
    pub is_primary: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerView {
    pub id: i64,
    pub customer_code: String,
    pub full_name: String,
    pub mobile_display: Option<String>,
    pub mobile_masked: Option<String>,
    pub aadhaar_masked: Option<String>,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pin_code: Option<String>,
    pub accounts: Vec<CustomerAccountView>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInput {
    pub full_name: String,
    pub mobile: String,
    pub aadhaar: Option<String>,
    pub bank_id: i64,
    pub account_number: String,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pin_code: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerInput {
    pub customer_id: i64,
    pub account_id: i64,
    pub full_name: String,
    pub mobile: String,
    pub aadhaar: Option<String>,
    pub bank_id: i64,
    pub account_number: String,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pin_code: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCustomerInput {
    pub query: String,
}

pub fn list_banks(connection: &Connection) -> AppResult<Vec<BankOption>> {
    let mut statement = connection
        .prepare("SELECT id, name, short_name FROM banks WHERE is_active = 1 ORDER BY name")?;
    let rows = statement.query_map([], |row| {
        Ok(BankOption {
            id: row.get(0)?,
            name: row.get(1)?,
            short_name: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn create_customer(
    connection: &mut Connection,
    input: CustomerInput,
    user_id: i64,
) -> AppResult<CustomerView> {
    let normalized = normalize_customer_input(input)?;
    let secret = get_or_create_app_secret(connection)?;
    let account_hash = security::hmac_lookup(&secret, &normalized.account_number)?;
    ensure_account_is_unique(connection, &account_hash, None)?;

    let aadhaar_hash = normalized
        .aadhaar
        .as_ref()
        .map(|aadhaar| security::hmac_lookup(&secret, aadhaar))
        .transpose()?;
    ensure_aadhaar_is_unique(connection, aadhaar_hash.as_deref(), None)?;

    let tx = connection.transaction()?;
    let customer_code = next_customer_code(&tx)?;
    let aadhaar_encrypted = normalized
        .aadhaar
        .as_ref()
        .map(|aadhaar| security::encrypt_sensitive(&secret, aadhaar))
        .transpose()?;
    let account_encrypted = security::encrypt_sensitive(&secret, &normalized.account_number)?;

    tx.execute(
        "INSERT INTO customers (
            customer_code, full_name, mobile_normalized, mobile_display, aadhaar_last4,
            aadhaar_encrypted, aadhaar_lookup_hash, address_line, city, state, pin_code,
            created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'), datetime('now'))",
        params![
            customer_code,
            normalized.full_name,
            normalized.mobile,
            normalized.mobile,
            normalized.aadhaar.as_deref().map(last4),
            aadhaar_encrypted,
            aadhaar_hash,
            normalized.address_line,
            normalized.city,
            normalized.state,
            normalized.pin_code,
        ],
    )?;
    let customer_id = tx.last_insert_rowid();

    tx.execute(
        "INSERT INTO customer_bank_accounts (
            customer_id, bank_id, account_number_encrypted, account_number_last4,
            account_lookup_hash, is_primary, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, 1, datetime('now'))",
        params![
            customer_id,
            normalized.bank_id,
            account_encrypted,
            last4(&normalized.account_number),
            account_hash,
        ],
    )?;

    tx.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, description, created_at)
         VALUES (?1, 'CUSTOMER_CREATED', 'CUSTOMER', ?2, 'Customer profile created.', datetime('now'))",
        params![user_id, customer_id],
    )?;
    tx.commit()?;

    get_customer(connection, customer_id)
}

pub fn update_customer(
    connection: &mut Connection,
    input: UpdateCustomerInput,
    user_id: i64,
) -> AppResult<CustomerView> {
    let normalized = normalize_customer_input(CustomerInput {
        full_name: input.full_name,
        mobile: input.mobile,
        aadhaar: input.aadhaar,
        bank_id: input.bank_id,
        account_number: input.account_number,
        address_line: input.address_line,
        city: input.city,
        state: input.state,
        pin_code: input.pin_code,
    })?;
    let secret = get_or_create_app_secret(connection)?;
    let account_hash = security::hmac_lookup(&secret, &normalized.account_number)?;
    ensure_account_is_unique(connection, &account_hash, Some(input.account_id))?;

    let aadhaar_hash = normalized
        .aadhaar
        .as_ref()
        .map(|aadhaar| security::hmac_lookup(&secret, aadhaar))
        .transpose()?;
    ensure_aadhaar_is_unique(connection, aadhaar_hash.as_deref(), Some(input.customer_id))?;

    let tx = connection.transaction()?;
    let exists: Option<i64> = tx
        .query_row(
            "SELECT id FROM customers WHERE id = ?1 AND is_active = 1",
            params![input.customer_id],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(AppError::Validation("Customer was not found.".to_string()));
    }

    let aadhaar_encrypted = normalized
        .aadhaar
        .as_ref()
        .map(|aadhaar| security::encrypt_sensitive(&secret, aadhaar))
        .transpose()?;
    let account_encrypted = security::encrypt_sensitive(&secret, &normalized.account_number)?;

    tx.execute(
        "UPDATE customers
         SET full_name = ?1, mobile_normalized = ?2, mobile_display = ?3,
             aadhaar_last4 = ?4, aadhaar_encrypted = ?5, aadhaar_lookup_hash = ?6,
             address_line = ?7, city = ?8, state = ?9, pin_code = ?10, updated_at = datetime('now')
         WHERE id = ?11",
        params![
            normalized.full_name,
            normalized.mobile,
            normalized.mobile,
            normalized.aadhaar.as_deref().map(last4),
            aadhaar_encrypted,
            aadhaar_hash,
            normalized.address_line,
            normalized.city,
            normalized.state,
            normalized.pin_code,
            input.customer_id,
        ],
    )?;

    tx.execute(
        "UPDATE customer_bank_accounts
         SET bank_id = ?1, account_number_encrypted = ?2, account_number_last4 = ?3,
             account_lookup_hash = ?4
         WHERE id = ?5 AND customer_id = ?6",
        params![
            normalized.bank_id,
            account_encrypted,
            last4(&normalized.account_number),
            account_hash,
            input.account_id,
            input.customer_id,
        ],
    )?;

    tx.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, description, created_at)
         VALUES (?1, 'CUSTOMER_UPDATED', 'CUSTOMER', ?2, 'Customer profile updated.', datetime('now'))",
        params![user_id, input.customer_id],
    )?;
    tx.commit()?;

    get_customer(connection, input.customer_id)
}

pub fn search_customer(
    connection: &Connection,
    input: SearchCustomerInput,
) -> AppResult<Vec<CustomerView>> {
    let query = input.query.trim();
    if query.len() < 4 {
        return Err(AppError::Validation(
            "Enter at least 4 characters to search.".to_string(),
        ));
    }

    let secret = get_or_create_app_secret_readonly(connection)?;
    let digits = digits_only(query);
    let account = normalize_account_number(query);
    let mut ids = Vec::<i64>::new();

    if let Ok(mobile) = normalize_mobile(&digits) {
        collect_ids(
            connection,
            "SELECT id FROM customers WHERE is_active = 1 AND mobile_normalized = ?1 LIMIT 10",
            &mobile,
            &mut ids,
        )?;
    } else if digits.len() >= 4 {
        collect_ids(
            connection,
            "SELECT id FROM customers WHERE is_active = 1 AND mobile_normalized LIKE ?1 LIMIT 10",
            &format!("{}%", digits),
            &mut ids,
        )?;
    }

    if digits.len() == 12 {
        let aadhaar_hash = security::hmac_lookup(&secret, &digits)?;
        collect_ids(
            connection,
            "SELECT id FROM customers WHERE is_active = 1 AND aadhaar_lookup_hash = ?1 LIMIT 10",
            &aadhaar_hash,
            &mut ids,
        )?;
    }

    if account.len() >= 4 {
        if account.len() >= 6 {
            let account_hash = security::hmac_lookup(&secret, &account)?;
            collect_ids(
                connection,
                "SELECT customer_id FROM customer_bank_accounts WHERE is_active = 1 AND account_lookup_hash = ?1 LIMIT 10",
                &account_hash,
                &mut ids,
            )?;
        }
        collect_ids(
            connection,
            "SELECT customer_id FROM customer_bank_accounts WHERE is_active = 1 AND account_number_last4 = ?1 LIMIT 10",
            &last4(&account),
            &mut ids,
        )?;
    }

    collect_ids(
        connection,
        "SELECT id FROM customers WHERE is_active = 1 AND customer_code = ?1 LIMIT 10",
        &query.to_uppercase(),
        &mut ids,
    )?;

    ids.sort_unstable();
    ids.dedup();
    ids.into_iter()
        .take(10)
        .map(|id| get_customer(connection, id))
        .collect()
}

pub fn get_customer(connection: &Connection, customer_id: i64) -> AppResult<CustomerView> {
    let secret = get_or_create_app_secret_readonly(connection)?;
    let mut customer = connection.query_row(
        "SELECT id, customer_code, full_name, mobile_display, aadhaar_last4,
                address_line, city, state, pin_code
         FROM customers
         WHERE id = ?1 AND is_active = 1",
        params![customer_id],
        |row| {
            let mobile_display: Option<String> = row.get(3)?;
            let aadhaar_last4: Option<String> = row.get(4)?;
            Ok(CustomerView {
                id: row.get(0)?,
                customer_code: row.get(1)?,
                full_name: row.get(2)?,
                mobile_masked: mobile_display.as_deref().map(mask_mobile),
                mobile_display,
                aadhaar_masked: aadhaar_last4.map(|value| format!("XXXX XXXX {}", value)),
                address_line: row.get(5)?,
                city: row.get(6)?,
                state: row.get(7)?,
                pin_code: row.get(8)?,
                accounts: Vec::new(),
            })
        },
    )?;

    let mut statement = connection.prepare(
        "SELECT a.id, a.bank_id, b.name, a.account_number_last4, a.account_number_encrypted, a.is_primary
         FROM customer_bank_accounts a
         JOIN banks b ON b.id = a.bank_id
         WHERE a.customer_id = ?1 AND a.is_active = 1
         ORDER BY a.is_primary DESC, a.id ASC",
    )?;
    let accounts = statement.query_map(params![customer_id], |row| {
        let last4: String = row.get(3)?;
        let encrypted_account: Vec<u8> = row.get(4)?;
        let account_display = security::decrypt_sensitive(&secret, &encrypted_account)
            .unwrap_or_else(|_| format!("XXXXXXXX{}", last4));
        Ok(CustomerAccountView {
            id: row.get(0)?,
            bank_id: row.get(1)?,
            bank_name: row.get(2)?,
            account_masked: format!("XXXXXXXX{}", last4),
            account_display,
            account_last4: last4,
            is_primary: row.get::<_, i64>(5)? == 1,
        })
    })?;
    customer.accounts = accounts.collect::<Result<Vec<_>, _>>()?;

    Ok(customer)
}

struct NormalizedCustomerInput {
    full_name: String,
    mobile: String,
    aadhaar: Option<String>,
    bank_id: i64,
    account_number: String,
    address_line: Option<String>,
    city: Option<String>,
    state: Option<String>,
    pin_code: Option<String>,
}

fn normalize_customer_input(input: CustomerInput) -> AppResult<NormalizedCustomerInput> {
    let full_name = input.full_name.trim().to_string();
    if full_name.len() < 2 || full_name.len() > 120 {
        return Err(AppError::Validation(
            "Customer name must be between 2 and 120 characters.".to_string(),
        ));
    }

    if input.bank_id <= 0 {
        return Err(AppError::Validation("Select a bank.".to_string()));
    }

    let mobile = normalize_mobile(&input.mobile)?;
    let aadhaar = input
        .aadhaar
        .as_deref()
        .map(digits_only)
        .filter(|value| !value.is_empty());
    if let Some(value) = aadhaar.as_deref() {
        if value.len() != 12 {
            return Err(AppError::Validation(
                "Aadhaar must contain 12 digits when provided.".to_string(),
            ));
        }
    }

    let account_number = normalize_account_number(&input.account_number);
    if !(6..=30).contains(&account_number.len()) {
        return Err(AppError::Validation(
            "Account number must be 6 to 30 letters or digits.".to_string(),
        ));
    }

    Ok(NormalizedCustomerInput {
        full_name,
        mobile,
        aadhaar,
        bank_id: input.bank_id,
        account_number,
        address_line: clean_optional(input.address_line),
        city: clean_optional(input.city),
        state: clean_optional(input.state),
        pin_code: clean_optional(input.pin_code),
    })
}

fn normalize_mobile(input: &str) -> AppResult<String> {
    let digits = digits_only(input);
    let mobile = if digits.len() == 12 && digits.starts_with("91") {
        digits[2..].to_string()
    } else {
        digits
    };

    if mobile.len() == 10 && matches!(mobile.as_bytes()[0], b'6'..=b'9') {
        Ok(mobile)
    } else {
        Err(AppError::Validation(
            "Mobile number must be a valid 10 digit Indian number.".to_string(),
        ))
    }
}

fn normalize_account_number(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

fn digits_only(input: &str) -> String {
    input.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn last4(value: &str) -> String {
    value
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

fn mask_mobile(value: &str) -> String {
    if value.len() == 10 {
        format!("{}XXXXXX{}", &value[0..2], &value[8..10])
    } else {
        "XXXXXXXXXX".to_string()
    }
}

fn next_customer_code(connection: &Connection) -> AppResult<String> {
    let next_id: i64 = connection.query_row(
        "SELECT COALESCE(MAX(id), 0) + 1 FROM customers",
        [],
        |row| row.get(0),
    )?;
    Ok(format!("CUST-{next_id:06}"))
}

fn get_or_create_app_secret(connection: &Connection) -> AppResult<String> {
    if let Some(secret) = settings_repository::get_value(connection, "app_secret")? {
        return Ok(secret);
    }

    let secret = security::generate_secret_hex();
    settings_repository::set_value(connection, "app_secret", &secret)?;
    Ok(secret)
}

fn get_or_create_app_secret_readonly(connection: &Connection) -> AppResult<String> {
    get_or_create_app_secret(connection)
}

fn ensure_account_is_unique(
    connection: &Connection,
    account_hash: &str,
    allowed_account_id: Option<i64>,
) -> AppResult<()> {
    let found: Option<i64> = connection
        .query_row(
            "SELECT id FROM customer_bank_accounts WHERE account_lookup_hash = ?1 AND is_active = 1 LIMIT 1",
            params![account_hash],
            |row| row.get(0),
        )
        .optional()?;

    if found.is_some() && found != allowed_account_id {
        return Err(AppError::Validation(
            "This account number is already linked to another customer.".to_string(),
        ));
    }
    Ok(())
}

fn ensure_aadhaar_is_unique(
    connection: &Connection,
    aadhaar_hash: Option<&str>,
    allowed_customer_id: Option<i64>,
) -> AppResult<()> {
    let Some(aadhaar_hash) = aadhaar_hash else {
        return Ok(());
    };
    let found: Option<i64> = connection
        .query_row(
            "SELECT id FROM customers WHERE aadhaar_lookup_hash = ?1 AND is_active = 1 LIMIT 1",
            params![aadhaar_hash],
            |row| row.get(0),
        )
        .optional()?;

    if found.is_some() && found != allowed_customer_id {
        return Err(AppError::Validation(
            "This Aadhaar number may already belong to another customer.".to_string(),
        ));
    }
    Ok(())
}

fn collect_ids(
    connection: &Connection,
    sql: &str,
    value: &str,
    ids: &mut Vec<i64>,
) -> AppResult<()> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map(params![value], |row| row.get::<_, i64>(0))?;
    for row in rows {
        ids.push(row?);
    }
    Ok(())
}

pub fn list_customers(connection: &Connection) -> AppResult<Vec<CustomerView>> {
    let mut statement = connection.prepare(
        "SELECT id FROM customers WHERE is_active = 1 ORDER BY updated_at DESC, id DESC LIMIT 500",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, i64>(0))?;
    let ids = rows.collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| get_customer(connection, id))
        .collect()
}
