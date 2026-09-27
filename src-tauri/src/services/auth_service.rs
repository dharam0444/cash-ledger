use crate::errors::{AppError, AppResult};
use crate::repositories::user_repository;
use crate::security;
use rusqlite::{params, Connection};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub id: i64,
    pub username: String,
    pub full_name: Option<String>,
    pub role: String,
}

pub fn ensure_default_admin(connection: &Connection) -> AppResult<()> {
    let user_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    if user_count > 0 {
        return Ok(());
    }

    let (password_hash, password_salt) = security::hash_password("admin123")?;
    connection.execute(
        "INSERT INTO users (username, password_hash, password_salt, full_name, role, created_at)
         VALUES (?1, ?2, ?3, 'Administrator', 'ADMIN', datetime('now'))",
        params!["admin", password_hash, password_salt],
    )?;
    connection.execute(
        "INSERT INTO audit_logs (action, entity_type, description, created_at)
         VALUES ('USER_CREATED', 'USER', 'Default admin user created during first launch.', datetime('now'))",
        [],
    )?;
    Ok(())
}

pub fn login(connection: &Connection, input: LoginInput) -> AppResult<AuthenticatedUser> {
    let username = input.username.trim();
    if username.is_empty() || input.password.is_empty() {
        return Err(AppError::Validation(
            "Username and password are required.".to_string(),
        ));
    }

    let Some(user) = user_repository::find_active_by_username(connection, username)? else {
        record_login(connection, None, false)?;
        return Err(AppError::InvalidCredentials);
    };

    if !security::verify_password(&input.password, &user.password_hash)? {
        record_login(connection, Some(user.id), false)?;
        return Err(AppError::InvalidCredentials);
    }

    record_login(connection, Some(user.id), true)?;
    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username,
        full_name: user.full_name,
        role: user.role,
    })
}

fn record_login(connection: &Connection, user_id: Option<i64>, success: bool) -> AppResult<()> {
    let action = if success {
        "LOGIN_SUCCESS"
    } else {
        "LOGIN_FAILED"
    };
    connection.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, description, created_at)
         VALUES (?1, ?2, 'USER', ?3, datetime('now'))",
        params![
            user_id,
            action,
            if success {
                "Operator login succeeded."
            } else {
                "Operator login failed."
            }
        ],
    )?;
    Ok(())
}
