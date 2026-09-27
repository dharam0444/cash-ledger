CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    password_salt TEXT,
    full_name TEXT,
    role TEXT NOT NULL DEFAULT 'ADMIN',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_code TEXT NOT NULL UNIQUE,
    full_name TEXT NOT NULL,
    mobile_normalized TEXT,
    mobile_display TEXT,
    aadhaar_last4 TEXT,
    aadhaar_encrypted BLOB,
    aadhaar_lookup_hash TEXT,
    address_line TEXT,
    city TEXT,
    state TEXT,
    pin_code TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    short_name TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS customer_bank_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    bank_id INTEGER NOT NULL,
    account_number_encrypted BLOB NOT NULL,
    account_number_last4 TEXT NOT NULL,
    account_lookup_hash TEXT NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    FOREIGN KEY(customer_id) REFERENCES customers(id),
    FOREIGN KEY(bank_id) REFERENCES banks(id)
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_number TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL,
    bank_account_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('DEPOSIT', 'WITHDRAWAL')),
    amount_paise INTEGER NOT NULL CHECK(amount_paise > 0),
    transaction_timestamp TEXT NOT NULL,
    remarks TEXT,
    created_by INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    created_at TEXT NOT NULL,
    FOREIGN KEY(customer_id) REFERENCES customers(id),
    FOREIGN KEY(bank_account_id) REFERENCES customer_bank_accounts(id),
    FOREIGN KEY(created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS application_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS backup_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backup_path TEXT NOT NULL,
    backup_type TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    verified_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_customer_mobile
ON customers(mobile_normalized);

CREATE INDEX IF NOT EXISTS idx_customer_aadhaar_hash
ON customers(aadhaar_lookup_hash);

CREATE INDEX IF NOT EXISTS idx_account_lookup_hash
ON customer_bank_accounts(account_lookup_hash);

CREATE INDEX IF NOT EXISTS idx_transaction_date
ON transactions(transaction_timestamp);

CREATE INDEX IF NOT EXISTS idx_transaction_customer
ON transactions(customer_id);

CREATE INDEX IF NOT EXISTS idx_transaction_type_date
ON transactions(transaction_type, transaction_timestamp);

INSERT OR IGNORE INTO application_settings (key, value, updated_at)
VALUES
    ('schema_version', '1', datetime('now')),
    ('shop_name', 'Cash Ledger', datetime('now')),
    ('automatic_backup_enabled', 'true', datetime('now')),
    ('backup_retention_days', '30', datetime('now')),
    ('auto_lock_minutes', '10', datetime('now'));

INSERT OR IGNORE INTO banks (name, short_name)
VALUES
    ('State Bank of India', 'SBI'),
    ('Bank of Baroda', 'BOB'),
    ('Punjab National Bank', 'PNB'),
    ('HDFC Bank', 'HDFC'),
    ('ICICI Bank', 'ICICI'),
    ('Bank of India', 'BOI'),
    ('Central Bank of India', 'CBI');
