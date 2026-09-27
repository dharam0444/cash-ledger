# Implementation Status

## Source Documents

- Read: `offline_cash_ledger_desktop_app_technical_design.md`
- Note: `/docs` does not exist in the current workspace, so there were no additional documents to read there.

## Architecture Decisions

- Kept the requested stack: Tauri 2, React, TypeScript, Vite, SQLite, Rust.
- Kept SQL access in the Rust/Tauri backend. React calls Tauri commands only.
- Used `rusqlite` for SQLite because the app is offline, single-computer, and command oriented.
- Used Argon2id-compatible `argon2` crate for password hashing. Passwords are not stored as plaintext.
- Added AES-256-GCM for sensitive field encryption and HMAC-SHA-256 lookup tokens for exact Aadhaar/account search.
- Live data is configured under the OS application data directory rather than beside the executable.

## Phase 1 — Foundation

Status: Complete.

Included:

- Tauri 2 application structure under `src-tauri/`
- React/TypeScript/Vite frontend under `src/`
- SQLite initial migration at `src-tauri/migrations/001_initial.sql`
- Database tables from the design: users, customers, banks, customer bank accounts, transactions, audit logs, application settings, backup history
- Required indexes for customer/account lookup and transaction reporting
- WAL and foreign key SQLite pragmas during database open
- Local login command: `auth_login`
- First-launch admin seeding with hashed default credentials
- Settings commands: `get_settings`, `update_settings`
- Basic login screen and dashboard shell

Default development credentials:

- Username: `admin`
- Password: `admin123`

Production note:

- The default password is for initial local bootstrap only and must be changed before production use.

## Phase 2 — Customer Management

Status: Complete.

Included:

- Bank dropdown loaded from the SQLite `banks` master table
- Customer create/edit form
- Required customer fields: name, mobile, bank, account number
- Optional fields: Aadhaar, address, city/village, state, PIN code
- Frontend Zod validation for customer form input
- Backend validation for name, mobile, Aadhaar format, bank, and account number
- Mobile/account/Aadhaar normalization before storage and search
- Full Aadhaar and account number encrypted before storage
- Aadhaar/account exact-search lookup tokens stored with HMAC-SHA-256
- Aadhaar, mobile, and account values masked in normal UI display
- Duplicate checks for account number and Aadhaar lookup token
- Universal search command: `search_customer`
- Customer detail command: `get_customer`
- Customer write commands: `create_customer`, `update_customer`
- Bank command: `list_banks`
- Audit logs for customer create/update
- In-memory migration test that verifies schema version and seeded bank records

Current scope note:

- The database supports multiple bank accounts per customer. The current UI creates/edits the primary account; a dedicated “add another account” workflow can be added later without changing the schema.

## Phase 3 — Transactions, Daily Ledger, Listings, End-Day Report

Status: Complete.

Included:

- Single-page-at-a-time UI with page tabs.
- Default page is Search Customer.
- Add Customer button placed beside Search.
- Add/Edit Customer opens as its own page, not beside the search page.
- Cash Deposit and Cash Withdrawal transaction form for the selected customer.
- Transaction save command: `create_transaction`.
- Immutable transaction row creation with database transaction and audit log.
- Human-readable transaction numbers in `TXN-YYYYMMDD-000001` format.
- Amounts stored as integer paise.
- Daily summary command: `get_daily_summary`.
- Daily transaction listing command: `get_daily_transactions`.
- End Day Report command: `get_daily_report`.
- Customer listing page using `list_customers`.
- Daily Cash Management listing page with deposit/withdrawal totals and ledger rows.
- Printable End Day Report page.
- Frontend transaction validation for type and amount.


## Latest UI/Report Refinements

Status: Complete.

- Removed State and PIN code from the Add/Edit Customer form.
- Cash Management and End Day Report now show full mobile number and full account number for admin/client printing needs.
- Added backend decryption for account numbers only in admin listing/report DTOs.
- Kept Aadhaar masked in normal UI/report output.
- Updated A4 print CSS for the End Day Report: black-and-white, no dark header, tighter row padding, print margins, and wrapping to reduce overlapping row data.


## Setup, Licensing, And Cloud Backup Status

Status: Setup/licensing foundation added. Google Drive backup not implemented yet.

Added:

- Setup page in the app.
- Client name and mobile fields.
- Admin password update during setup.
- Machine-bound fingerprint code generation.
- Offline license key activation flow.
- License status command and local storage in application settings.

Important security note:

- The current license algorithm is an offline foundation suitable for normal client control, but before commercial release the vendor secret must be changed and ideally replaced with public/private key signing.

Google Drive backup plan:

- Use Google Drive rather than Gmail attachments for backup storage.
- Use Google OAuth Desktop App credentials.
- Upload encrypted .clbak files to a Drive folder such as CashLedgerBackup/YYYY/MM/.
- App must continue working fully offline if Google sync fails.
- Required from developer before implementation: Google Cloud OAuth Desktop Client ID/secret and consent configuration.

## Verification

Completed:

- `cargo fmt` passed.
- `cargo test` passed.
- `npm run build` passed.
- `npm test` passed.
- `npm audit --omit=dev` passed with 0 production vulnerabilities.
- Existing Tauri dev server is running on port 1420.
- Live app data exists under `/home/hb/.local/share/com.cashledger.offline/data/`.
- Migration verification passed through Rust in-memory SQLite test.

Environment note:

- In this shell, `npm run tauri:dev` needs Rust in PATH. Run `source ~/.cargo/env` first, or run with `PATH=/home/hb/.cargo/bin:$PATH npm run tauri:dev`.
- A second startup attempt will fail if another dev server is already listening on port 1420. Stop the existing app first, or keep using the running app.

## Next Phase

Phase 4 can broaden reports: date-range report, CSV export, customer-wise report filters, and PDF/print polish.
