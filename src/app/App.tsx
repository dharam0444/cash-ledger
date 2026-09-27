import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LoginPage } from "../pages/Login/LoginPage";
import { DashboardPage } from "../pages/Transaction/DashboardPage";
import type { AppSettings, AuthenticatedUser, DbStatus } from "../types";

type BootState =
  | { status: "loading" }
  | { status: "ready"; dbStatus: DbStatus; settings: AppSettings }
  | { status: "error"; message: string };

function formatAppDate(date: Date) {
  return new Intl.DateTimeFormat("en-IN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
  })
    .format(date)
    .toLowerCase();
}

export function App() {
  const [bootState, setBootState] = useState<BootState>({ status: "loading" });
  const [user, setUser] = useState<AuthenticatedUser | null>(null);
  const [appDate, setAppDate] = useState(() => formatAppDate(new Date()));

  useEffect(() => {
    async function boot() {
      try {
        const dbStatus = await invoke<DbStatus>("initialize_app");
        const settings = await invoke<AppSettings>("get_settings");
        setBootState({ status: "ready", dbStatus, settings });
      } catch (error) {
        setBootState({
          status: "error",
          message: error instanceof Error ? error.message : String(error),
        });
      }
    }

    void boot();
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setAppDate(formatAppDate(new Date()));
    }, 1000);

    return () => window.clearInterval(timer);
  }, []);

  if (bootState.status === "loading") {
    return <main className="screen-center">Starting Cash Ledger...</main>;
  }

  if (bootState.status === "error") {
    return (
      <main className="screen-center error-panel">
        <h1>Unable to Start</h1>
        <p>{bootState.message}</p>
      </main>
    );
  }

  if (!user) {
    return (
      <LoginPage
        appDate={appDate}
        dbStatus={bootState.dbStatus}
        settings={bootState.settings}
        onLogin={setUser}
      />
    );
  }

  return (
    <DashboardPage
      appDate={appDate}
      user={user}
      settings={bootState.settings}
      onLogout={() => setUser(null)}
    />
  );
}
