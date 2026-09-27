use crate::errors::AppResult;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub role: String,
}

pub fn find_active_by_username(
    connection: &Connection,
    username: &str,
) -> AppResult<Option<UserRecord>> {
    let mut statement = connection.prepare(
        "SELECT id, username, password_hash, full_name, role
         FROM users
         WHERE username = ?1 AND is_active = 1",
    )?;

    let mut rows = statement.query(params![username])?;
    if let Some(row) = rows.next()? {
        Ok(Some(UserRecord {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            full_name: row.get(3)?,
            role: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}
