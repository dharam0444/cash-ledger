# Offline Cash Deposit & Withdrawal Ledger Desktop Application
## Technical Requirement & System Design Document

**Document version:** 1.0  
**Target platform:** Windows Desktop (offline-first)  
**Application type:** Single-shop / single-computer desktop application  
**Primary purpose:** Replace the manually maintained customer cash deposit/withdrawal ledger with a secure, fast and lightweight desktop application.

---

# 1. Project Overview

The client operates an online banking kiosk / customer service center where customers regularly visit for banking-related cash deposit and cash withdrawal transactions.

Currently the operator manually maintains a physical ledger book and records details such as:

- Date
- Time
- Customer name
- Bank name
- Aadhaar number
- Customer account number
- Customer address
- Customer mobile number
- Cash deposit amount
- Cash withdrawal amount

The proposed application will digitize this process while remaining completely usable without internet access.

The application should:

1. Run smoothly on an ordinary Windows desktop/laptop.
2. Store all business data locally.
3. Search existing customers extremely quickly.
4. Avoid re-entering customer details for every transaction.
5. Allow transaction entry in a few seconds.
6. Produce daily/date-range reports.
7. Maintain local backups.
8. Optionally upload encrypted backups using the client's Google account.
9. Protect sensitive customer information.
10. Consume significantly less RAM/CPU than a normal browser-heavy desktop application.

---

# 2. Recommended Product Flow

The entire daily operation should revolve around one main screen.

## Main operator flow

```text
Open Application
      |
      v
Operator Login
      |
      v
Main Transaction Screen
      |
      v
Enter Search Value
(Mobile / Aadhaar / Account Number)
      |
      +--------------------------+
      |                          |
      v                          v
Customer Found              Customer Not Found
      |                          |
      v                          v
Show Customer Details       Show "Add Customer"
      |                          |
      |                          v
      |                    Add Customer Details
      |                          |
      +-------------+------------+
                    |
                    v
             Select Transaction
          Deposit / Withdrawal
                    |
                    v
              Enter Amount
                    |
                    v
            Review & Save
                    |
                    v
             Transaction Done
```

The operator should normally complete a returning customer's transaction using only:

1. Search customer
2. Select Deposit or Withdrawal
3. Enter amount
4. Save

This keeps repetitive data entry to a minimum.

---

# 3. Core Functional Requirements

## 3.1 Login

The application should open with a local operator login.

Minimum fields:

- Username
- Password

Optional future support:

- Admin
- Operator
- Read-only/report user

For the first version, a single admin/operator account is sufficient.

### Security

Never store the password as plain text.

Store:

```text
password_hash
password_salt
```

Use a strong password hashing algorithm such as Argon2id.

---

# 4. Main Transaction Screen

This is the most important screen.

The page should be optimized for keyboard entry.

## Suggested layout

```text
+--------------------------------------------------------------------+
| CASH LEDGER                                      26 Sep 2026 10:40 |
+--------------------------------------------------------------------+

 Search Customer
 [ Mobile / Aadhaar / Account Number __________________ ] [ Search ]

 --------------------------------------------------------------------

 Customer Details

 Name             : Ramesh Kumar
 Mobile           : 98XXXXXX21
 Aadhaar          : XXXX XXXX 1234
 Bank             : State Bank of India
 Account          : XXXXXXXX5412
 Address          : Biaora, Madhya Pradesh

 --------------------------------------------------------------------

 Transaction Type

 ( ) Cash Deposit
 ( ) Cash Withdrawal

 Amount: ₹ [____________]

 Remarks: [____________________________________________]

                       [ SAVE TRANSACTION ]

 --------------------------------------------------------------------

 Today
 Deposits    : ₹ 45,000
 Withdrawals : ₹ 31,500
 Customers   : 26
 Transactions: 31
 --------------------------------------------------------------------
```

---

# 5. Universal Customer Search

The operator should not need separate search fields.

Use a single search box.

Example input:

```text
9876543210
```

The application should automatically find whether it matches:

- mobile number
- Aadhaar value / Aadhaar lookup token
- account number
- optional customer ID

## Recommended behaviour

Start searching after:

- user presses Enter, or
- user enters at least 4-5 characters

For numeric identifiers, exact match should be preferred.

Possible result behaviour:

```text
Exact customer found
=> immediately load profile
```

If several accounts share a mobile number:

```text
Show small result list
=> operator selects correct customer
```

---

# 6. Existing Customer Flow

When customer exists:

The application should display customer data as read-only.

Operator normally should NOT need to modify:

- name
- Aadhaar
- mobile
- account number
- bank
- address

Only allow:

```text
Transaction Type:
- Deposit
- Withdrawal

Amount:
₹ XXXXX

Optional Remarks
```

Provide a separate:

```text
Edit Customer
```

button for corrections.

This avoids accidental profile changes during a transaction.

---

# 7. New Customer Flow

If no customer matches:

```text
Customer not found

[ + ADD NEW CUSTOMER ]
```

Clicking Add Customer opens a form.

## Fields

### Required

- Customer name
- Mobile number
- Bank
- Account number

### Aadhaar

Aadhaar should be treated as highly sensitive information.

Recommended product behaviour:

- collect it only where the client has a valid business/legal requirement;
- encrypt any stored full value;
- never show the complete Aadhaar on normal screens;
- display only the final four digits;
- avoid including the full number in reports/exports.

Example display:

```text
XXXX XXXX 4821
```

### Optional

- Address
- Village/city
- PIN code
- Remarks

## Validation examples

Mobile:

```regex
^[6-9][0-9]{9}$
```

Account number:

- digits/alphanumeric depending upon bank
- configurable length
- no hard assumption that all banks use same length

Aadhaar:

- validate format/checksum if collected
- never rely on Aadhaar itself as the database primary key

---

# 8. Transaction Entry

Every transaction should create a new immutable ledger entry.

## Transaction types

```text
DEPOSIT
WITHDRAWAL
```

Database value:

```text
D
W
```

or an enum.

## Transaction fields

- transaction_id
- transaction_number
- customer_id
- transaction_type
- amount
- transaction_date
- transaction_time / timestamp
- bank_account_id
- remarks
- created_by
- created_at
- updated_at
- status

### Important rule

Do NOT update previous transactions when a new deposit or withdrawal occurs.

Every action creates a separate ledger row.

This creates a reliable transaction history.

---

# 9. Recommended Database

## SQLite

Use **SQLite** as the local database.

Reasons:

- no server installation
- almost zero administration
- extremely small footprint
- excellent performance for one computer
- supports indexes and transactions
- entire database can live in one local file
- easy encrypted backup
- perfect for an offline desktop application
- handles far more records than this kiosk is likely to create

A PostgreSQL/MySQL server is unnecessary for the first offline single-computer version.

---

# 10. Database Schema

Recommended logical schema:

```text
users
customers
banks
customer_bank_accounts
transactions
audit_logs
application_settings
backup_history
```

---

# 11. Users Table

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    password_salt TEXT,
    full_name TEXT,
    role TEXT NOT NULL DEFAULT 'ADMIN',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
```

---

# 12. Customers Table

Do not use Aadhaar as the primary key.

```sql
CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    customer_code TEXT NOT NULL UNIQUE,

    full_name TEXT NOT NULL,

    mobile_normalized TEXT,
    mobile_display TEXT,

    aadhaar_last4 TEXT,

    -- encrypted full value only if legally/business required
    aadhaar_encrypted BLOB,

    -- keyed lookup token / HMAC for exact search
    aadhaar_lookup_hash TEXT,

    address_line TEXT,
    city TEXT,
    state TEXT,
    pin_code TEXT,

    is_active INTEGER NOT NULL DEFAULT 1,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## Why store an Aadhaar lookup hash?

If the application must allow searching by full Aadhaar but should not index the raw Aadhaar value:

```text
Entered Aadhaar
     |
     v
Normalize
     |
     v
HMAC-SHA-256
     |
     v
Search aadhaar_lookup_hash
```

This allows fast equality search without storing an indexed plaintext Aadhaar number.

The actual number, where legally required, can be separately encrypted.

---

# 13. Banks Table

```sql
CREATE TABLE banks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    short_name TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
);
```

Examples:

```text
State Bank of India
Bank of Baroda
Punjab National Bank
HDFC Bank
ICICI Bank
Bank of India
Central Bank of India
```

Use an autocomplete/dropdown.

---

# 14. Customer Bank Accounts

A customer may have more than one bank account.

Therefore account information should preferably NOT be placed directly inside the customer table.

```sql
CREATE TABLE customer_bank_accounts (
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
```

This enables:

```text
One customer
    |
    +-- SBI Account
    +-- Bank of Baroda Account
    +-- HDFC Account
```

---

# 15. Transactions Table

Money must never use floating point.

Use integer **paise**.

Example:

```text
₹1250.50
```

store as:

```text
125050
```

Schema:

```sql
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    transaction_number TEXT NOT NULL UNIQUE,

    customer_id INTEGER NOT NULL,
    bank_account_id INTEGER NOT NULL,

    transaction_type TEXT NOT NULL
        CHECK(transaction_type IN ('DEPOSIT', 'WITHDRAWAL')),

    amount_paise INTEGER NOT NULL
        CHECK(amount_paise > 0),

    transaction_timestamp TEXT NOT NULL,

    remarks TEXT,

    created_by INTEGER NOT NULL,

    status TEXT NOT NULL DEFAULT 'ACTIVE',

    created_at TEXT NOT NULL,

    FOREIGN KEY(customer_id) REFERENCES customers(id),
    FOREIGN KEY(bank_account_id) REFERENCES customer_bank_accounts(id),
    FOREIGN KEY(created_by) REFERENCES users(id)
);
```

---

# 16. Transaction Number

Generate a human-readable unique transaction number.

Example:

```text
TXN-20260926-000001
TXN-20260926-000002
TXN-20260926-000003
```

Do not use this as the internal database primary key.

---

# 17. Audit Log

Because this is financial record keeping, changes should be auditable.

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    user_id INTEGER,

    action TEXT NOT NULL,

    entity_type TEXT NOT NULL,

    entity_id INTEGER,

    description TEXT,

    created_at TEXT NOT NULL
);
```

Examples:

```text
CUSTOMER_CREATED
CUSTOMER_UPDATED
TRANSACTION_CREATED
TRANSACTION_VOIDED
BACKUP_CREATED
LOGIN_SUCCESS
LOGIN_FAILED
REPORT_EXPORTED
```

Never store passwords or complete Aadhaar values inside audit descriptions.

---

# 18. Transaction Editing Policy

Recommended rule:

## Do not directly edit completed financial transactions.

If operator enters a wrong transaction:

```text
Original transaction
Status = VOID
```

Then create a corrected transaction.

Store:

- void reason
- voided_by
- voided_at

This protects ledger history.

---

# 19. Database Indexes

Fast search comes mostly from correct indexes.

```sql
CREATE INDEX idx_customer_mobile
ON customers(mobile_normalized);

CREATE INDEX idx_customer_aadhaar_hash
ON customers(aadhaar_lookup_hash);

CREATE INDEX idx_account_lookup_hash
ON customer_bank_accounts(account_lookup_hash);

CREATE INDEX idx_transaction_date
ON transactions(transaction_timestamp);

CREATE INDEX idx_transaction_customer
ON transactions(customer_id);

CREATE INDEX idx_transaction_type_date
ON transactions(transaction_type, transaction_timestamp);
```

For this application, a properly indexed SQLite database should make customer lookups effectively instantaneous even with hundreds of thousands of records on a normal office PC.

---

# 20. Search Strategy

Normalize values before search.

## Mobile normalization

Input:

```text
+91 98765-43210
```

normalize:

```text
9876543210
```

## Aadhaar

Input:

```text
1234 5678 9012
```

normalize:

```text
123456789012
```

Then derive the lookup HMAC.

## Account Number

Remove:

- spaces
- hyphens
- accidental formatting characters

Convert alphabetic characters to uppercase where appropriate.

---

# 21. Recommended Desktop Technology Stack

## Recommended option

```text
UI                 React + TypeScript
Build Tool         Vite
Desktop Runtime    Tauri 2
Native Backend     Rust / Tauri commands
Database           SQLite
Database Access    SQLx or rusqlite
Validation         Zod (frontend) + backend validation
Encryption         OS-backed key + modern authenticated encryption
Password Hash      Argon2id
Reports            HTML -> Print / PDF
Backup             encrypted database backup
Installer          MSI / NSIS installer
```

## Why Tauri instead of Electron?

Electron embeds Chromium + Node.js and normally consumes more memory.

Tauri uses the operating system webview and a small native backend.

For this project that means:

- smaller application size
- lower RAM consumption
- lower idle CPU usage
- fast startup
- good React development experience
- local native file/database access

Because this app needs only forms, search, reports and a local database, Tauri is a strong fit.

---

# 22. Alternative If Team Does Not Want Rust

Possible alternatives:

### Option A

```text
.NET 8/9 + WPF or WinUI
SQLite
```

Very suitable for Windows-only deployment.

### Option B

```text
Flutter Desktop
SQLite
```

Good UI and cross-platform support.

### Option C

```text
Electron + React
SQLite
```

Easiest for many JavaScript developers but heavier.

## Preferred order for this requirement

```text
Tauri + React + SQLite
or
.NET + SQLite
```

Avoid Electron unless development familiarity is more important than minimum memory use.

---

# 23. Suggested Application Architecture

```text
┌─────────────────────────────────────────────┐
│                React UI                     │
│                                             │
│ Customer Search                             │
│ Customer Form                               │
│ Transaction Form                            │
│ Reports                                     │
│ Settings                                    │
└──────────────────────┬──────────────────────┘
                       │
                 Tauri Commands
                       │
┌──────────────────────▼──────────────────────┐
│          Application / Service Layer        │
│                                             │
│ Customer Service                            │
│ Transaction Service                         │
│ Report Service                              │
│ Backup Service                              │
│ Encryption Service                          │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────▼──────────────────────┐
│             Repository Layer                │
│                                             │
│ CustomerRepository                          │
│ TransactionRepository                       │
│ AccountRepository                           │
│ AuditRepository                             │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────▼──────────────────────┐
│                    SQLite                   │
│               Local Encrypted Data          │
└─────────────────────────────────────────────┘
```

Do not let React execute arbitrary raw SQL.

All important operations should pass through the native application/service layer.

---

# 24. Suggested Project Structure

```text
cash-ledger/
│
├── src/
│   ├── app/
│   ├── components/
│   ├── pages/
│   │   ├── Login/
│   │   ├── Transaction/
│   │   ├── Customers/
│   │   ├── Reports/
│   │   └── Settings/
│   │
│   ├── features/
│   │   ├── customers/
│   │   ├── transactions/
│   │   ├── reports/
│   │   └── backups/
│   │
│   ├── hooks/
│   ├── schemas/
│   ├── utils/
│   └── types/
│
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── services/
│   │   ├── repositories/
│   │   ├── security/
│   │   ├── backup/
│   │   ├── reports/
│   │   └── main.rs
│   │
│   ├── migrations/
│   └── tauri.conf.json
│
└── package.json
```

---

# 25. Screens

Keep the first release small.

## Screen 1 — Login

```text
Username
Password
Login
```

## Screen 2 — Dashboard / Transaction Entry

Main daily working screen.

Contains:

- universal customer search
- customer details
- Add Customer
- transaction type
- amount
- Save Transaction
- today's totals

## Screen 3 — Customer Form

Add/edit customer.

## Screen 4 — Customers

Search and view customer transaction history.

## Screen 5 — Reports

Daily and date-range reporting.

## Screen 6 — Backup & Settings

- local backup
- restore backup
- cloud/email backup
- change password
- shop settings
- bank master data

No unnecessary complex menus.

---

# 26. Dashboard Statistics

Useful values:

```text
Today's Deposit Total
Today's Withdrawal Total
Today's Transaction Count
Today's Unique Customers
Net Cash Movement
```

Formula:

```text
Net Cash Movement =
Deposits - Withdrawals
```

Do not call this number "profit".

It is only the net transaction cash movement unless the business separately tracks service charges/commission.

---

# 27. Reports

## Daily Report

Input:

```text
Date: 26-09-2026
```

Output:

```text
Opening/optional cash figure
Total Deposit
Total Withdrawal
Net Movement
No. of Deposit Transactions
No. of Withdrawal Transactions
Total Transactions
Unique Customers
```

Detailed table:

```text
Time
Transaction No
Customer
Mobile
Bank
Account Last 4
Type
Amount
Operator
```

---

# 28. Date Range Report

Examples:

```text
Today
Yesterday
This Week
This Month
Custom Range
```

Custom:

```text
From: 01-09-2026
To:   26-09-2026
```

---

# 29. Customer Report

Search one customer and show:

```text
Customer Details

Total Deposit
Total Withdrawal
Total Transactions

Transaction History
```

Filter:

- date range
- bank
- transaction type

---

# 30. Report Export

Support:

- print
- PDF
- CSV
- Excel later if required

Privacy rule:

Reports should normally show:

```text
Mobile: 98XXXXXX21
Aadhaar: XXXX XXXX 1234
Account: XXXXXXXX5412
```

Avoid full sensitive identifiers.

---

# 31. Backup Architecture

Do not depend only upon the live SQLite file.

Recommended backup flow:

```text
SQLite Live Database
        |
        v
SQLite Online Backup / Snapshot
        |
        v
Compress
        |
        v
Encrypt Backup
        |
        +----------------------+
        |                      |
        v                      v
Local Backup Folder       Optional Google Backup
```

Example:

```text
CashLedger_Backup_2026-09-26_220500.clbak
```

---

# 32. Backup Frequency

Recommended:

```text
Automatic local backup:
- on application close
- once per day after first transaction
- retain multiple historical copies

Optional:
- manual "Backup Now"
```

Retention example:

```text
Daily backups   = 30 days
Monthly backups = 12 months
```

Do not silently delete the only known-good backup.

---

# 33. Google / Gmail Backup

The client's statement "backup in Gmail ID" should preferably be implemented using the Google account rather than treating Gmail as the primary database.

## Better option

Upload the encrypted `.clbak` file to:

```text
Google Drive
```

using Google OAuth.

Advantages:

- intended for file storage
- organized backup folder
- easier restore
- less clutter than sending email attachments

Example Drive structure:

```text
CashLedgerBackup/
  2026/
    09/
      CashLedger_Backup_2026-09-26.clbak
```

## Alternative

The application may email the encrypted backup to the client's own Gmail account.

However Google Drive is the recommended cloud-backup destination.

### Important

The application must remain completely usable if:

```text
Internet unavailable
Google authentication unavailable
Google API unavailable
```

Cloud backup should never block transaction entry.

---

# 34. Backup Encryption

Cloud backup must never contain a plain SQLite database if it contains sensitive identifiers.

Recommended design:

```text
Database snapshot
      |
      v
Archive
      |
      v
AES-256-GCM / equivalent authenticated encryption
      |
      v
Encrypted .clbak
```

The encryption key should not simply be hardcoded in source code.

Possible key strategy:

- app master secret protected by Windows Credential Manager / DPAPI
- password-derived recovery key for backup restore

A restore process must be tested before production launch.

A backup that has never been restored successfully should not be considered a verified backup.

---

# 35. Local Database Security

SQLite by itself is a file format, not a complete security boundary.

Recommended protections:

1. Windows user-account file permissions.
2. Application authentication.
3. Encrypt sensitive fields.
4. Consider an encrypted SQLite solution where appropriate.
5. Never log full Aadhaar/account information.
6. Mask sensitive values in the UI.
7. Lock application after inactivity.
8. Maintain audit logs.
9. Disable unrestricted developer tools in production.
10. Code-sign the Windows installer if practical.

---

# 36. Sensitive-Field Encryption

Suggested sensitive values:

```text
Full Aadhaar
Full account number
Potentially full mobile number depending on business policy
```

Pattern:

```text
plaintext
  |
  v
encrypt()
  |
  v
ciphertext saved to database
```

For exact search:

```text
normalized plaintext
  |
  v
HMAC(secret, value)
  |
  v
lookup hash
```

This is preferable to a simple unsalted SHA hash for low-entropy identifiers because an HMAC requires the application's secret key for lookup-token generation.

---

# 37. Aadhaar & Privacy Considerations

This application handles personally identifiable and financial information.

Before production use, the client should confirm what customer information it is legally required and permitted to collect for its kiosk/banking relationship.

UIDAI materials define Masked Aadhaar as showing only the last four digits, and UIDAI regulations impose restrictions/requirements around Aadhaar collection, storage and use.

Therefore the application should implement:

- masking by default;
- minimum necessary collection;
- encryption;
- access control;
- no Aadhaar in logs;
- no full Aadhaar in normal reports;
- retention/deletion rules;
- backup protection.

India's Digital Personal Data Protection framework should also be considered when defining notices, purpose, retention, security safeguards and handling of personal data.

This design document is a technical plan, not a substitute for legal/compliance advice from the relevant bank/BC/CSP network or qualified advisor.

---

# 38. Data Validation

## Customer name

- required
- trim spaces
- length limits

## Mobile

- numeric
- normalize to 10 digits for Indian numbers
- optional duplicate warning

A duplicate mobile number should not automatically be blocked because family members may legitimately share one contact number.

## Aadhaar

If collected:

- normalize
- validate expected structure/checksum
- check lookup hash for existing record
- display masked

## Account number

- normalize
- validate minimum/maximum
- detect duplicate account via lookup hash

## Amount

Rules:

```text
> 0
maximum configurable
max 2 decimal places
```

Always store paise as an integer.

---

# 39. Duplicate Customer Prevention

When adding a customer, check:

```text
Aadhaar lookup hash
Account number lookup hash
Mobile
```

If matching information exists:

```text
Possible existing customer found
```

Show the operator the masked candidate.

Do not automatically merge records.

---

# 40. Transaction Atomicity

Saving a financial transaction must use a database transaction.

Pseudo-flow:

```text
BEGIN

validate user
validate customer
validate bank account
validate transaction
insert transaction
insert audit log

COMMIT
```

If anything fails:

```text
ROLLBACK
```

This prevents half-saved records.

---

# 41. Concurrency

For the initial requirement:

```text
1 desktop
1 application
1 operator at a time
```

SQLite is ideal.

If the client later asks:

```text
3-10 computers simultaneously
```

do not simply place the SQLite file on a shared network folder.

At that stage migrate to:

```text
central server API
+
PostgreSQL
```

while preserving the desktop UI if desired.

---

# 42. Performance Goals

Target:

```text
App launch                  < 2 seconds on normal SSD system
Exact customer search       < 100 ms typical
Save transaction            < 200 ms typical
Daily report                < 500 ms typical
```

Actual performance depends on hardware.

Avoid:

- loading all customer rows into React
- querying every transaction on startup
- large JavaScript libraries
- unnecessary animations
- continuously running timers
- storing photos/scans in the database unless required

Use pagination for long history tables.

---

# 43. Recommended UI Principles

The operator performs repetitive entry all day.

Therefore:

### Keyboard first

Example:

```text
Ctrl + N    New Customer
Ctrl + T    New Transaction
Ctrl + R    Reports
Ctrl + B    Backup
Esc         Cancel / Close dialog
Enter       Search / Continue
```

### Auto-focus

After completing transaction:

```text
Search field automatically focused
```

Operator can immediately serve next customer.

### Large amount input

Make amount prominent.

### Confirmation

Before saving:

```text
Ramesh Kumar
SBI XXXX5412
Cash Deposit
₹15,000

Confirm?
```

This can prevent costly mistakes.

---

# 44. Receipt (Optional Feature)

After transaction save:

```text
Print Receipt
```

Receipt can include:

- shop name
- transaction number
- timestamp
- customer name
- masked account
- transaction type
- amount
- operator

Do not show full Aadhaar.

This can be added in Phase 2 if not required initially.

---

# 45. Application Settings

Possible settings:

```text
Shop Name
Owner Name
Address
Phone
Logo
Report Header
Currency
Automatic Backup Enabled
Google Backup Enabled
Backup Retention Days
Auto-lock Time
```

---

# 46. Error Handling

Messages should be understandable.

Bad:

```text
SQLITE_CONSTRAINT_FOREIGNKEY
```

Good:

```text
Unable to save the transaction.
The selected customer account is no longer active.
```

Maintain technical logs separately.

Sensitive values must be redacted.

---

# 47. Crash Recovery

SQLite should run in a safe journaling mode such as WAL where appropriate.

Important design:

```text
Save transaction
=> commit database transaction
=> show success
```

Never show "Transaction Saved" before the commit succeeds.

---

# 48. Application Update Strategy

Because the application works offline, updates can initially be distributed as a signed installer.

Example:

```text
CashLedgerSetup-1.0.0.exe
CashLedgerSetup-1.1.0.exe
```

Database schema updates must use migrations.

Never require deleting the database during upgrade.

---

# 49. Database Migration Strategy

Folder:

```text
migrations/
  001_initial.sql
  002_add_void_reason.sql
  003_add_backup_history.sql
```

Maintain a schema version.

Before applying major migration:

```text
Create automatic backup
Apply migration
Verify
```

---

# 50. Restore Flow

Settings:

```text
Backup & Restore
```

Operator selects:

```text
Restore Backup
```

Process:

```text
Authenticate admin
Select .clbak
Validate backup
Decrypt
Verify database integrity
Create safety backup of current database
Restore
Restart application
```

A restore operation should require admin authentication.

---

# 51. Suggested Daily Report SQL

Example concept:

```sql
SELECT
    transaction_type,
    COUNT(*) AS transaction_count,
    SUM(amount_paise) AS total_paise
FROM transactions
WHERE transaction_timestamp >= :start
  AND transaction_timestamp < :end
  AND status = 'ACTIVE'
GROUP BY transaction_type;
```

Unique customers:

```sql
SELECT COUNT(DISTINCT customer_id)
FROM transactions
WHERE transaction_timestamp >= :start
  AND transaction_timestamp < :end
  AND status = 'ACTIVE';
```

Use start/end timestamps rather than applying string formatting functions to the indexed column where possible.

---

# 52. Example API / Command Interfaces

Even though this is a local desktop application, create clear internal service boundaries.

```ts
type SearchCustomerInput = {
  query: string;
};

type CreateTransactionInput = {
  customerId: number;
  bankAccountId: number;
  type: "DEPOSIT" | "WITHDRAWAL";
  amountPaise: number;
  remarks?: string;
};
```

Commands:

```text
auth_login
search_customer
get_customer
create_customer
update_customer
create_transaction
void_transaction
get_daily_summary
get_report
create_backup
restore_backup
```

---

# 53. React State Management

This application does not need a large state-management framework initially.

Use:

```text
React state
React Context for authenticated user/settings
TanStack Query optionally for command request caching
```

Avoid adding Redux unless application complexity later justifies it.

---

# 54. Form Validation

Recommended:

```text
React Hook Form
+
Zod
```

But frontend validation is only for user experience.

All critical validation must also occur in the Tauri/Rust service layer.

Never trust frontend data for financial transactions.

---

# 55. Report Pagination

For large transaction histories:

```text
Page size: 50 / 100
```

Query:

```sql
ORDER BY transaction_timestamp DESC
LIMIT :limit OFFSET :offset
```

For very large databases, cursor-based pagination can later replace OFFSET pagination.

---

# 56. Database File Location

Do not store live data next to the executable.

Use an application data directory similar to:

```text
%LOCALAPPDATA%/CashLedger/
```

Suggested structure:

```text
CashLedger/
  data/
    ledger.db

  backups/
    ...

  logs/
    application.log

  config/
    settings.json
```

Protect permissions appropriately.

---

# 57. Application Lock

Recommended:

```text
Auto Lock = 5 / 10 / 15 minutes
```

After inactivity:

```text
Locked
Password required
```

This is important because customer information is visible on the computer.

---

# 58. Data Retention

Retention should not be hardcoded without understanding banking/CSP and legal requirements.

Make retention configurable only after confirming applicable requirements.

The business should document:

```text
why each data field is collected
how long it is retained
who can access it
when it is deleted
how backups expire
```

---

# 59. MVP Scope

Version 1 should contain:

- local login
- customer create/edit
- universal search
- multiple customer bank accounts
- deposit entry
- withdrawal entry
- transaction history
- daily report
- date-range report
- PDF/print report
- local backup
- restore
- masked sensitive data
- audit logging
- installer

This is enough to replace the paper ledger.

---

# 60. Phase 2 Features

After MVP is stable:

- Google Drive backup
- encrypted email backup
- thermal receipt printing
- Excel export
- commission/service-charge tracking
- multiple operators
- role permissions
- customer photo (only if truly required)
- dashboard charts
- automatic software update
- multiple shop/branch support
- SMS/WhatsApp receipt through an approved integration if needed

---

# 61. Features I Would NOT Put in MVP

Avoid making the first version unnecessarily complex.

Do not initially add:

- microservices
- Docker
- Kubernetes
- Redis
- separate backend server
- PostgreSQL
- cloud database
- real-time WebSocket
- Redux unless needed
- analytics platform
- unnecessary AI

For one offline kiosk these technologies provide little benefit and add failure points.

---

# 62. Recommended Development Plan

## Stage 1 — Foundation

- Tauri project
- React/TypeScript
- SQLite
- migrations
- login
- settings

## Stage 2 — Customer Management

- customer table
- account table
- customer create/edit
- validation
- universal search
- indexes
- masking/encryption

## Stage 3 — Transactions

- deposit
- withdrawal
- transaction number
- validation
- audit log
- transaction history

## Stage 4 — Reports

- daily summary
- date-range report
- customer report
- PDF/print
- CSV export

## Stage 5 — Backup

- local database snapshot
- encryption
- retention
- restore
- integrity verification

## Stage 6 — Google Backup

- OAuth login
- Drive upload
- backup history
- retry when internet returns
- manual backup test

## Stage 7 — Production Hardening

- installer
- logging
- recovery testing
- performance test
- permission review
- security review
- database migration testing

---

# 63. Testing Checklist

## Customer

- create customer
- invalid mobile
- duplicate account
- duplicate Aadhaar lookup
- multiple accounts
- edit profile

## Search

- mobile
- Aadhaar
- account number
- whitespace
- hyphens
- no result
- multiple mobile matches

## Transaction

- deposit
- withdrawal
- zero amount
- negative amount
- very large amount
- duplicate rapid click
- app crash during save
- void transaction

## Reports

- no transaction day
- one transaction
- hundreds of transactions
- date boundary
- month boundary
- totals match manual calculation

## Backup

- manual backup
- automatic backup
- restore
- corrupt backup
- wrong password/key
- low disk space
- internet unavailable
- Google upload failure

---

# 64. Security Acceptance Checklist

Before delivery:

```text
[ ] Password not stored as plain text
[ ] Aadhaar masked in UI
[ ] Aadhaar absent from logs
[ ] Account number masked in reports
[ ] Sensitive values encrypted
[ ] Backup encrypted
[ ] Database access restricted
[ ] SQL parameters used everywhere
[ ] Audit trail enabled
[ ] Restore requires authentication
[ ] Production devtools disabled
[ ] App locks after inactivity
[ ] Backup restore actually tested
```

---

# 65. Suggested Final Architecture

```text
                    WINDOWS DESKTOP
┌────────────────────────────────────────────────────────┐
│                                                        │
│  Tauri Application                                     │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │ React + TypeScript UI                            │  │
│  │                                                  │  │
│  │ Search -> Customer -> Deposit/Withdrawal        │  │
│  │ Reports -> Backup -> Settings                   │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │                               │
│                 Tauri IPC Commands                    │
│                        │                               │
│  ┌─────────────────────▼────────────────────────────┐  │
│  │ Rust Application Layer                          │  │
│  │                                                 │  │
│  │ Validation                                      │  │
│  │ Encryption                                      │  │
│  │ Transaction Service                             │  │
│  │ Reporting                                       │  │
│  │ Backup                                          │  │
│  │ Audit                                           │  │
│  └─────────────────────┬────────────────────────────┘  │
│                        │                               │
│  ┌─────────────────────▼────────────────────────────┐  │
│  │ SQLite Local Database                           │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
└────────────────────────────────────────────────────────┘
                         |
                         | Optional Internet
                         v
                  ┌───────────────┐
                  │ Google Drive  │
                  │ Encrypted     │
                  │ Backup Only   │
                  └───────────────┘
```

---

# 66. Final Recommendation

For this client's exact requirement, the recommended implementation is:

```text
Tauri 2
+
React
+
TypeScript
+
SQLite
```

with a native service layer responsible for all sensitive database and security operations.

This architecture gives the project:

- offline-first operation
- low RAM use
- low CPU use
- very fast customer search
- small deployment footprint
- no local database server
- simple maintenance
- secure local data model
- future cloud-backup capability
- straightforward path to a server architecture if the business later becomes multi-PC or multi-branch

The most important product decision is to keep the daily workflow extremely simple:

```text
Search customer
      ↓
Select Deposit / Withdrawal
      ↓
Enter amount
      ↓
Save
```

Returning customers should not require their profile information to be typed again.

---

# 67. Important Terminology

Throughout the UI use **Cash**, not **Case**.

Correct terms:

```text
Cash Deposit
Cash Withdrawal
Cash Ledger
Cash In
Cash Out
```

---

# 68. Regulatory References Reviewed for This Design

- UIDAI — Aadhaar Online Services / definition of Masked Aadhaar:
  https://uidai.gov.in/en/921-faqs/aadhaar-online.html

- UIDAI — Aadhaar Authentication and Offline Verification Regulations:
  https://uidai.gov.in/en/about-uidai/legal-framework/updated-regulation.html

- Ministry of Electronics and Information Technology — Digital Personal Data Protection Rules, 2025:
  https://www.meity.gov.in/documents/act-and-policies/digital-personal-data-protection-rules-2025-gDOxUjMtQWa

These sources should be rechecked before production deployment, particularly if the kiosk is operating under a bank, Business Correspondent, CSP, AePS, or another regulated banking arrangement.

---

# 69. Next Technical Deliverables

After accepting this architecture, the development repository can be broken into these concrete documents:

```text
01-requirements.md
02-database-schema.md
03-ui-flow.md
04-security-design.md
05-backup-restore.md
06-api-command-contracts.md
07-test-cases.md
08-release-deployment.md
```

This single document can be used as the initial master system-design specification.
