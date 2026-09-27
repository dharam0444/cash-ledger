import { FormEvent, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { loginSchema } from "../../schemas/login";
import type { AppSettings, AuthenticatedUser, DbStatus } from "../../types";

type Props = {
  appDate: string;
  dbStatus: DbStatus;
  settings: AppSettings;
  onLogin: (user: AuthenticatedUser) => void;
};

export function LoginPage({ dbStatus, onLogin }: Props) {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);

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

          <button className="sober-login-button" type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Signing in..." : "Login"}
          </button>
        </form>

        <p className="login-version">v{dbStatus.schemaVersion}</p>
      </section>
    </main>
  );
}
