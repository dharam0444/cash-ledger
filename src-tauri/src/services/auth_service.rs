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

pub fn ensure_default_admin(_connection: &Connection) -> AppResult<()> {
    // The first admin is created by the first-run setup screen. Never create a
    // predictable default password here.
    Ok(())
}

pub fn reset_admin_password(
    connection: &Connection,
    machine_code: &str,
    recovery_code: &str,
    new_password: &str,
) -> AppResult<()> {
    if !crate::services::license_service::validate_recovery_code(machine_code, recovery_code) {
        return Err(AppError::Validation(
            "Invalid or expired recovery code.".to_string(),
        ));
    }
    let normalized_code = recovery_code
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();
    let was_used: i64 = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM used_recovery_codes WHERE code = ?1)",
        params![&normalized_code],
        |row| row.get(0),
    )?;
    if was_used != 0 {
        return Err(AppError::Validation(
            "This recovery code has already been used.".to_string(),
        ));
    }
    let password = new_password.trim();
    if password.len() < 6 {
        return Err(AppError::Validation(
            "Password must be at least 6 characters.".to_string(),
        ));
    }
    let (hash, salt) = security::hash_password(password)?;
    let updated = connection.execute(
        "UPDATE users SET password_hash = ?1, password_salt = ?2 WHERE username = 'admin' AND role = 'ADMIN'",
        params![hash, salt],
    )?;
    if updated == 0 {
        return Err(AppError::Validation(
            "Admin account has not been set up yet.".to_string(),
        ));
    }
    connection.execute(
        "INSERT INTO audit_logs (action, entity_type, description, created_at) VALUES ('PASSWORD_RESET', 'USER', 'Admin password reset using recovery code.', datetime('now'))",
        [],
    )?;
    connection.execute(
        "INSERT INTO used_recovery_codes (code, used_at) VALUES (?1, datetime('now'))",
        params![normalized_code],
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

#[cfg(test)]
mod tests {
    use super::reset_admin_password;
    use crate::errors::AppError;
    use crate::recovery_code::generate_recovery_code;
    use crate::security;
    use rusqlite::{params, Connection};

    #[test]
    fn each_recovery_code_can_be_redeemed_only_once() {
        let connection = Connection::open_in_memory().expect("in-memory database opens");
        connection
            .execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("schema initializes");
        let (hash, salt) = security::hash_password("initial-password").expect("password hashes");
        connection
            .execute(
                "INSERT INTO users (username, password_hash, password_salt, role, created_at)
                 VALUES ('admin', ?1, ?2, 'ADMIN', datetime('now'))",
                params![hash, salt],
            )
            .expect("admin is created");

        let machine_code = "2169F3-34E2DE-B3281F";
        let code = generate_recovery_code(machine_code, [8, 7, 6, 5, 4, 3, 2, 1]);
        reset_admin_password(&connection, machine_code, &code, "new-password")
            .expect("first redemption succeeds");

        let error = reset_admin_password(&connection, machine_code, &code, "another-password")
            .expect_err("second redemption is rejected");
        assert!(
            matches!(error, AppError::Validation(message) if message.contains("already been used"))
        );
    }
}
