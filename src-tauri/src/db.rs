use crate::errors::{AppError, AppResult};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const INITIAL_MIGRATION: &str = include_str!("../migrations/001_initial.sql");

pub struct Database {
    connection: Mutex<Connection>,
    path: PathBuf,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    pub database_path: String,
    pub schema_version: i64,
    pub user_count: i64,
}

impl Database {
    pub fn open(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .or_else(|_| fallback_data_dir())
            .map_err(|_| AppError::DataDirectoryUnavailable)?;

        fs::create_dir_all(data_dir.join("data"))?;
        fs::create_dir_all(data_dir.join("backups"))?;
        fs::create_dir_all(data_dir.join("logs"))?;
        fs::create_dir_all(data_dir.join("config"))?;

        let path = data_dir.join("data").join("ledger.db");
        let connection = Connection::open(&path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self {
            connection: Mutex::new(connection),
            path,
        })
    }

    pub fn initialize(&self) -> AppResult<()> {
        let connection = self.lock()?;
        connection.execute_batch(INITIAL_MIGRATION)?;
        Ok(())
    }

    pub fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| AppError::Validation("Database lock is unavailable.".to_string()))
    }

    pub fn status(&self) -> AppResult<DbStatus> {
        let connection = self.lock()?;
        let schema_version: i64 = connection
            .query_row(
                "SELECT value FROM application_settings WHERE key = 'schema_version'",
                [],
                |row| row.get::<_, String>(0),
            )?
            .parse()
            .unwrap_or(1);
        let user_count =
            connection.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;

        Ok(DbStatus {
            database_path: self.path.display().to_string(),
            schema_version,
            user_count,
        })
    }
}

fn fallback_data_dir() -> Result<PathBuf, std::io::Error> {
    let project_dirs = ProjectDirs::from("com", "cashledger", "CashLedger")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "project dirs"))?;
    Ok(project_dirs.data_local_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::INITIAL_MIGRATION;
    use rusqlite::Connection;

    #[test]
    fn initial_migration_creates_schema_version_and_core_tables() {
        let connection = Connection::open_in_memory().expect("in-memory sqlite opens");
        connection
            .execute_batch(INITIAL_MIGRATION)
            .expect("initial migration runs");

        let schema_version: String = connection
            .query_row(
                "SELECT value FROM application_settings WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .expect("schema version exists");
        let bank_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM banks", [], |row| row.get(0))
            .expect("banks table exists");

        assert_eq!(schema_version, "1");
        assert!(bank_count >= 7);
    }
}
