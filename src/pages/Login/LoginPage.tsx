import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { loginSchema } from "../../schemas/login";
import type { AppSettings, AuthenticatedUser, DbStatus, LicenseStatus } from "../../types";

type Props = {
  appDate: string;
  dbStatus: DbStatus;
  settings: AppSettings;
  onLogin: (user: AuthenticatedUser) => void;
  onSetupComplete: () => Promise<void>;
};

export function LoginPage({ dbStatus, onLogin, onSetupComplete }: Props) {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showRecovery, setShowRecovery] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [machineCode, setMachineCode] = useState("");
  const [recoveryCode, setRecoveryCode] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [setup, setSetup] = useState({ clientName: "", clientMobile: "", adminPassword: "", licenseKey: "" });
  const [setupError, setSetupError] = useState<string | null>(null);

  useEffect(() => {
    if (dbStatus.userCount === 0) {
      void invoke<LicenseStatus>("get_license_status").then(status => setMachineCode(status.machineCode));
    }
  }, [dbStatus.userCount]);

  async function openRecovery() {
    setShowRecovery(true);
    setError(null);
    setSuccessMessage(null);
    const status = await invoke<LicenseStatus>("get_license_status");
    setMachineCode(status.machineCode);
  }

  async function handleRecovery(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    try {
      await invoke("reset_admin_password", { input: { machineCode, recoveryCode, newPassword } });
      setShowRecovery(false);
      setRecoveryCode("");
      setNewPassword("");
      setPassword("");
      setSuccessMessage("Password reset successfully. You can now log in.");
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  async function handleSetup(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSetupError(null);
    try {
      await invoke("complete_client_setup", { input: { ...setup, defaultBankId: null } });
      await onSetupComplete();
    } catch (err) { setSetupError(err instanceof Error ? err.message : String(err)); }
  }

  if (dbStatus.userCount === 0) {
    return <main className="login-shell sober-login-shell"><section className="login-panel sober-login-panel">
      <h1 className="sober-login-title">Cash Ledger Setup</h1>
      <p>Send this machine code to the administrator to receive a license key:</p>
      <strong className="setup-code-box">{machineCode || "Loading..."}</strong>
      <form className="sober-login-form" onSubmit={handleSetup}>
        <label>Client name<input required value={setup.clientName} onChange={e => setSetup({ ...setup, clientName: e.target.value })} /></label>
        <label>Mobile number<input required value={setup.clientMobile} onChange={e => setSetup({ ...setup, clientMobile: e.target.value })} /></label>
        <label>Admin password<input required minLength={6} type="password" value={setup.adminPassword} onChange={e => setSetup({ ...setup, adminPassword: e.target.value })} /></label>
        <label>License key<input required value={setup.licenseKey} onChange={e => setSetup({ ...setup, licenseKey: e.target.value.toUpperCase() })} placeholder="XXXX-XXXX-XXXX-XXXX" /></label>
        {setupError ? <p className="form-error">{setupError}</p> : null}
        <button className="sober-login-button" type="submit">Complete setup</button>
      </form>
      <p className="login-version">The machine code is unique to this computer.</p>
    </section></main>;
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setSuccessMessage(null);

    const parsed = loginSchema.safeParse({ username, password });
    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? "Enter valid login details.");
      return;
    }

    setIsSubmitting(true);
    try {
      const user = await invoke<AuthenticatedUser>("auth_login", {
        input: parsed.data,
      });
      onLogin(user);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="login-shell sober-login-shell">
      <section className="login-panel sober-login-panel" aria-labelledby="login-title">
        <h1 id="login-title" className="sober-login-title">Cash Ledger</h1>

        {showRecovery ? (
          <form className="sober-login-form recovery-panel" onSubmit={handleRecovery}>
            <h2>Reset admin password</h2>
            <p className="muted-text">Send this machine code to the administrator:</p>
            <strong className="setup-code-box">{machineCode}</strong>
            <label>Recovery code<input autoFocus required value={recoveryCode} onChange={e => setRecoveryCode(e.target.value.toUpperCase())} /></label>
            <label>New password<input required minLength={6} type="password" value={newPassword} onChange={e => setNewPassword(e.target.value)} /></label>
            {error ? <p className="form-error">{error}</p> : null}
            <button className="sober-login-button" type="submit">Reset password</button>
            <button type="button" className="text-button" onClick={() => { setShowRecovery(false); setError(null); }}>Back to login</button>
          </form>
        ) : (
          <form className="sober-login-form" onSubmit={handleSubmit}>
            <label>
              Username
              <input
                autoFocus
                autoComplete="username"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
              />
            </label>
            <label>
              Password
              <input
                autoComplete="current-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </label>

            {error ? <p className="form-error">{error}</p> : null}
            {successMessage ? <p className="success-note">{successMessage}</p> : null}

            <button className="sober-login-button" type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Signing in..." : "Login"}
            </button>
            <button type="button" className="text-button forgot-password-link" onClick={() => void openRecovery()}>Forgot password?</button>
          </form>
        )}

        <p className="login-version">v{dbStatus.schemaVersion}</p>
      </section>
    </main>
  );
}
