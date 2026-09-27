use crate::errors::AppResult;
use rusqlite::{params, Connection};

pub fn get_value(connection: &Connection, key: &str) -> AppResult<Option<String>> {
    let mut statement =
        connection.prepare("SELECT value FROM application_settings WHERE key = ?1")?;
    let mut rows = statement.query(params![key])?;
    Ok(rows.next()?.map(|row| row.get(0)).transpose()?)
}

pub fn set_value(connection: &Connection, key: &str, value: &str) -> AppResult<()> {
    connection.execute(
        "INSERT INTO application_settings (key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value],
    )?;
    Ok(())
}
