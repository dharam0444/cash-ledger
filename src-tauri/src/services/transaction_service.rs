use crate::errors::{AppError, AppResult};
use crate::repositories::settings_repository;
use crate::security;
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionInput {
    pub customer_id: i64,
    pub bank_account_id: i64,
    pub transaction_type: String,
    pub amount_paise: i64,
    pub remarks: Option<String>,
    pub created_by: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyQueryInput {
    pub date: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionView {
    pub id: i64,
    pub transaction_number: String,
    pub customer_id: i64,
    pub customer_name: String,
    pub mobile_display: Option<String>,
    pub mobile_masked: Option<String>,
    pub aadhaar_display: Option<String>,
    pub bank_name: String,
    pub account_display: String,
    pub account_masked: String,
    pub transaction_type: String,
    pub amount_paise: i64,
    pub transaction_timestamp: String,
    pub remarks: Option<String>,
    pub status: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySummary {
    pub date: String,
    pub deposit_total_paise: i64,
    pub withdrawal_total_paise: i64,
    pub deposit_count: i64,
    pub withdrawal_count: i64,
    pub transaction_count: i64,
    pub unique_customers: i64,
    pub net_movement_paise: i64,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyReport {
    pub summary: DailySummary,
    pub transactions: Vec<TransactionView>,
}

pub fn create_transaction(
    connection: &mut Connection,
    input: CreateTransactionInput,
) -> AppResult<TransactionView> {
    let transaction_type = input.transaction_type.trim().to_uppercase();
    if transaction_type != "DEPOSIT" && transaction_type != "WITHDRAWAL" {
        return Err(AppError::Validation(
            "Select cash deposit or cash withdrawal.".to_string(),
        ));
    }
    if input.amount_paise <= 0 {
        return Err(AppError::Validation(
            "Amount must be greater than zero.".to_string(),
        ));
    }
    if input.amount_paise > 100_000_000_00 {
        return Err(AppError::Validation(
            "Amount is above the allowed limit.".to_string(),
        ));
    }

    let remarks = input
        .remarks
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let now = Local::now();
    let transaction_date = now.format("%Y%m%d").to_string();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = connection.transaction()?;
    let account_owner: Option<i64> = tx
        .query_row(
            "SELECT customer_id FROM customer_bank_accounts WHERE id = ?1 AND is_active = 1",
            params![input.bank_account_id],
            |row| row.get(0),
        )
        .optional()?;
    if account_owner != Some(input.customer_id) {
        return Err(AppError::Validation(
            "The selected customer account is no longer active.".to_string(),
        ));
    }

    let customer_active: Option<i64> = tx
        .query_row(
            "SELECT id FROM customers WHERE id = ?1 AND is_active = 1",
            params![input.customer_id],
            |row| row.get(0),
        )
        .optional()?;
    if customer_active.is_none() {
        return Err(AppError::Validation("Customer was not found.".to_string()));
    }

    let sequence: i64 = tx.query_row(
        "SELECT COUNT(*) + 1 FROM transactions
         WHERE transaction_timestamp >= ?1 AND transaction_timestamp < ?2",
        params![
            now.format("%Y-%m-%d 00:00:00").to_string(),
            (now + Duration::days(1))
                .format("%Y-%m-%d 00:00:00")
                .to_string()
        ],
        |row| row.get(0),
    )?;
    let transaction_number = format!("TXN-{transaction_date}-{sequence:06}");

    tx.execute(
        "INSERT INTO transactions (
            transaction_number, customer_id, bank_account_id, transaction_type,
            amount_paise, transaction_timestamp, remarks, created_by, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
        params![
            transaction_number,
            input.customer_id,
            input.bank_account_id,
            transaction_type,
            input.amount_paise,
            timestamp,
            remarks,
            input.created_by,
        ],
    )?;
    let transaction_id = tx.last_insert_rowid();

    tx.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, description, created_at)
         VALUES (?1, 'TRANSACTION_CREATED', 'TRANSACTION', ?2, 'Ledger transaction created.', datetime('now'))",
        params![input.created_by, transaction_id],
    )?;
    tx.commit()?;

    get_transaction(connection, transaction_id)
}

pub fn get_daily_summary(
    connection: &Connection,
    input: DailyQueryInput,
) -> AppResult<DailySummary> {
    let (start, end) = date_bounds(&input.date)?;
    let mut summary = DailySummary {
        date: input.date,
        deposit_total_paise: 0,
        withdrawal_total_paise: 0,
        deposit_count: 0,
        withdrawal_count: 0,
        transaction_count: 0,
        unique_customers: 0,
        net_movement_paise: 0,
    };

    let mut statement = connection.prepare(
        "SELECT transaction_type, COUNT(*), COALESCE(SUM(amount_paise), 0)
         FROM transactions
         WHERE transaction_timestamp >= ?1 AND transaction_timestamp < ?2 AND status = 'ACTIVE'
         GROUP BY transaction_type",
    )?;
    let rows = statement.query_map(params![start, end], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    for row in rows {
        let (transaction_type, count, total) = row?;
        if transaction_type == "DEPOSIT" {
            summary.deposit_count = count;
            summary.deposit_total_paise = total;
        } else if transaction_type == "WITHDRAWAL" {
            summary.withdrawal_count = count;
            summary.withdrawal_total_paise = total;
        }
    }

    summary.transaction_count = summary.deposit_count + summary.withdrawal_count;
    summary.unique_customers = connection.query_row(
        "SELECT COUNT(DISTINCT customer_id)
         FROM transactions
         WHERE transaction_timestamp >= ?1 AND transaction_timestamp < ?2 AND status = 'ACTIVE'",
        params![start, end],
        |row| row.get(0),
    )?;
    summary.net_movement_paise = summary.deposit_total_paise - summary.withdrawal_total_paise;
    Ok(summary)
}

pub fn get_daily_transactions(
    connection: &Connection,
    input: DailyQueryInput,
) -> AppResult<Vec<TransactionView>> {
    let (start, end) = date_bounds(&input.date)?;
    list_transactions_between(connection, &start, &end)
}

pub fn get_daily_report(connection: &Connection, input: DailyQueryInput) -> AppResult<DailyReport> {
    let date = input.date.clone();
    Ok(DailyReport {
        summary: get_daily_summary(connection, DailyQueryInput { date: date.clone() })?,
        transactions: get_daily_transactions(connection, DailyQueryInput { date })?,
    })
}

fn get_transaction(connection: &Connection, transaction_id: i64) -> AppResult<TransactionView> {
    let secret = report_secret(connection)?;
    let sql = transaction_sql("WHERE t.id = ?1");
    let mut statement = connection.prepare(&sql)?;
    let mut rows = statement.query(params![transaction_id])?;
    let row = rows
        .next()?
        .ok_or_else(|| AppError::Validation("Transaction was not found.".to_string()))?;
    map_transaction(row, &secret)
}

fn list_transactions_between(
    connection: &Connection,
    start: &str,
    end: &str,
) -> AppResult<Vec<TransactionView>> {
    let secret = report_secret(connection)?;
    let sql = transaction_sql(
        "WHERE t.transaction_timestamp >= ?1 AND t.transaction_timestamp < ?2 AND t.status = 'ACTIVE'
         ORDER BY t.transaction_timestamp DESC, t.id DESC",
    );
    let mut statement = connection.prepare(&sql)?;
    let mut rows = statement.query(params![start, end])?;
    let mut transactions = Vec::new();
    while let Some(row) = rows.next()? {
        transactions.push(map_transaction(row, &secret)?);
    }
    Ok(transactions)
}

fn transaction_sql(where_clause: &str) -> String {
    format!(
        "SELECT t.id, t.transaction_number, t.customer_id, c.full_name, c.mobile_display,
                c.aadhaar_encrypted, b.name, a.account_number_last4, a.account_number_encrypted, t.transaction_type,
                t.amount_paise, t.transaction_timestamp, t.remarks, t.status
         FROM transactions t
         JOIN customers c ON c.id = t.customer_id
         JOIN customer_bank_accounts a ON a.id = t.bank_account_id
         JOIN banks b ON b.id = a.bank_id
         {where_clause}"
    )
}

fn map_transaction(row: &rusqlite::Row<'_>, secret: &str) -> AppResult<TransactionView> {
    let mobile: Option<String> = row.get(4)?;
    let encrypted_aadhaar: Option<Vec<u8>> = row.get(5)?;
    let account_last4: String = row.get(7)?;
    let encrypted_account: Vec<u8> = row.get(8)?;
    let account_display = security::decrypt_sensitive(secret, &encrypted_account)
        .unwrap_or_else(|_| format!("XXXXXXXX{}", account_last4));
    Ok(TransactionView {
        id: row.get(0)?,
        transaction_number: row.get(1)?,
        customer_id: row.get(2)?,
        customer_name: row.get(3)?,
        mobile_masked: mobile.as_deref().map(mask_mobile),
        mobile_display: mobile,
        aadhaar_display: encrypted_aadhaar
            .as_deref()
            .and_then(|value| security::decrypt_sensitive(secret, value).ok()),
        bank_name: row.get(6)?,
        account_masked: format!("XXXXXXXX{}", account_last4),
        account_display,
        transaction_type: row.get(9)?,
        amount_paise: row.get(10)?,
        transaction_timestamp: row.get(11)?,
        remarks: row.get(12)?,
        status: row.get(13)?,
    })
}

fn report_secret(connection: &Connection) -> AppResult<String> {
    settings_repository::get_value(connection, "app_secret")?
        .ok_or_else(|| AppError::Validation("Application secret is not initialized.".to_string()))
}

fn date_bounds(date: &str) -> AppResult<(String, String)> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Date must be in YYYY-MM-DD format.".to_string()))?;
    let next = parsed
        .checked_add_signed(Duration::days(1))
        .ok_or_else(|| AppError::Validation("Invalid report date.".to_string()))?;
    Ok((
        format!("{} 00:00:00", parsed.format("%Y-%m-%d")),
        format!("{} 00:00:00", next.format("%Y-%m-%d")),
    ))
}

fn mask_mobile(value: &str) -> String {
    if value.len() == 10 {
        format!("{}XXXXXX{}", &value[0..2], &value[8..10])
    } else {
        "XXXXXXXXXX".to_string()
    }
}
