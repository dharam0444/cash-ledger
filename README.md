# Cash Ledger

Offline desktop cash ledger built with Tauri 2, React, TypeScript, Vite, SQLite, and Rust.

## Development Commands

Use the same commands on Ubuntu and Windows unless a platform-specific command is shown.

### Install dependencies

Ubuntu/macOS/Git Bash:

```bash
npm install
```

Windows PowerShell:

```powershell
npm install
```

### Run the desktop app in development

Ubuntu/macOS/Git Bash:

```bash
npm run tauri:dev
```

Windows PowerShell:

```powershell
npm run tauri:dev
```

Tauri starts the local Vite dev server automatically using:

```bash
npm run dev
```

The dev server is fixed to:

```text
http://127.0.0.1:1420
```

Do not change `beforeDevCommand` to `npm.cmd`. `npm run dev` works on both Ubuntu and Windows.

### Run only the frontend dev server

Ubuntu/macOS/Git Bash:

```bash
npm run dev:frontend
```

Windows PowerShell:

```powershell
npm run dev:frontend
```

## Testing Commands

### Build frontend

```bash
npm run build
```

### Run frontend tests

```bash
npm test
```

### Run Rust tests

Ubuntu/macOS/Git Bash:

```bash
cd src-tauri
cargo test
cd ..
```

Windows PowerShell:

```powershell
cd src-tauri
cargo test
cd ..
```

## Build Desktop Installer/App

Ubuntu builds a Linux app. Windows builds the Windows `.exe` installer.

Ubuntu/macOS/Git Bash:

```bash
npm run tauri:build
```

Windows PowerShell:

```powershell
npm run tauri:build
```

On Windows, the installer is generated inside:

```text
src-tauri\target\release\bundle\nsis\
```

Look for a file similar to:

```text
Cash Ledger_0.1.0_x64-setup.exe
```

## First-Time Client Setup And Activation

When the app is installed for the first time, the setup screen shows a machine fingerprint code.

Client flow:

1. Open the app.
2. Copy the machine fingerprint code from the setup screen.
3. Send that code to the software owner/admin.
4. Owner/admin generates a license key using the command below.
5. Client enters client name, mobile number, admin password, default bank, and license key.
6. Submit setup to activate the app.

### Generate License Key For Setup

Run this from the project root.

Ubuntu/macOS/Git Bash:

```bash
node tools/generate-license-key.cjs MACHINE-CODE-HERE
```

Example:

```bash
node tools/generate-license-key.cjs 2169F3-34E2DE-B3281F
```

Windows PowerShell:

```powershell
node tools\generate-license-key.cjs MACHINE-CODE-HERE
```

Example:

```powershell
node tools\generate-license-key.cjs 2169F3-34E2DE-B3281F
```

The command prints a license key like:

```text
ABCD1234-EFGH5678-IJKL9012-MNOP3456
```

Give that license key to the client for the setup screen.

### Vendor Secret Note

Before final release, replace the development vendor secret with your private production secret and keep it consistent in the license and recovery code paths.

Do not publish the production secret in README, GitHub, screenshots, or client documents.

Files to review before release:

```text
tools/generate-license-key.cjs
src-tauri/src/services/license_service.rs
src-tauri/src/recovery_code.rs
```

If the secrets do not match, generated license keys or recovery codes will not work.

## Forgot Password Recovery

If admin forgets the password:

1. On the login screen, click `Forgot password?`.
2. The app shows the machine code.
3. Client sends the machine code to the software owner/admin.
4. Owner/admin generates a recovery code using the command below.
5. Client enters the recovery code and new password.
6. After successful reset, the app returns to the login screen.

### Generate Forgot Password Recovery Code

Run this from `src-tauri`.

Ubuntu/macOS/Git Bash:

```bash
cd src-tauri
cargo run --bin recovery-code -- MACHINE-CODE-HERE
cd ..
```

Example:

```bash
cd src-tauri
cargo run --bin recovery-code -- 2169F3-34E2DE-B3281F
cd ..
```

Windows PowerShell:

```powershell
cd src-tauri
cargo run --bin recovery-code -- MACHINE-CODE-HERE
cd ..
```

Example:

```powershell
cd src-tauri
cargo run --bin recovery-code -- 2169F3-34E2DE-B3281F
cd ..
```

The command prints a recovery code like:

```text
ABCD1234-EFGH5678-IJKL9012-MNOP3456
```

Each command generates a new, machine-specific recovery code. Give it to the client; every
redeemed code is retained locally and can be used only once on that installation.

## Useful Manual Test Checklist

After installing the app on a test Windows machine:

1. Open the app.
2. Complete setup with machine code and license key.
3. Login as admin.
4. Add customer.
5. Search customer.
6. Save cash deposit.
7. Save cash withdrawal.
8. Check Cash Management list.
9. Check End Day Report.
10. Print report preview.
11. Close and reopen app.
12. Confirm data is still available.
13. Test Forgot password recovery flow.

## Required Tools

### Ubuntu

Install:

- Node.js LTS
- Rust/Cargo via rustup
- Tauri Linux system dependencies

### Windows

Install:

- Node.js LTS
- Rust/Cargo via rustup
- Microsoft Visual Studio Build Tools with `Desktop development with C++`
- WebView2 Runtime

Install Rust/Cargo on Windows with PowerShell:

```powershell
winget install --id Rustlang.Rustup -e
```

Check Rust/Cargo:

```powershell
cargo --version
rustc --version
```

Install Visual Studio Build Tools:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e
```

Then open Visual Studio Installer and select:

```text
Desktop development with C++
```
