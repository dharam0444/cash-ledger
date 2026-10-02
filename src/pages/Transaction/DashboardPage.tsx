import { FormEvent, useEffect, useMemo, useState, type ChangeEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { customerSchema, type CustomerFormValues } from "../../schemas/customer";
import { amountToPaise, transactionSchema } from "../../schemas/transaction";
import type {
  AppSettings,
  AuthenticatedUser,
  BankOption,
  Customer,
  CustomerInput,
  DailyReport,
  DailySummary,
  Transaction,
  TransactionType,
  UpdateCustomerInput,
  LicenseStatus,
} from "../../types";

type Props = {
  appDate: string;
  user: AuthenticatedUser;
  settings: AppSettings;
  onLogout: () => void;
};

type Page = "search" | "customer-form" | "customers" | "cash-management" | "report" | "setup";
type FormMode = "create" | "edit";

type CustomerFormState = {
  fullName: string;
  mobile: string;
  aadhaar: string;
  bankId: string;
  accountNumber: string;
  addressLine: string;
  city: string;
};

type TransactionFormState = {
  transactionType: TransactionType;
  amount: string;
  remarks: string;
};

const emptyForm: CustomerFormState = {
  fullName: "",
  mobile: "",
  aadhaar: "",
  bankId: "",
  accountNumber: "",
  addressLine: "",
  city: "",
};

const emptyTransactionForm: TransactionFormState = {
  transactionType: "DEPOSIT",
  amount: "",
  remarks: "",
};

export function DashboardPage({ appDate, user, settings, onLogout }: Props) {
  const today = getLocalDateInputValue();
  const [page, setPage] = useState<Page>("search");
  const [banks, setBanks] = useState<BankOption[]>([]);
  const [banksError, setBanksError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<Customer[]>([]);
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null);
  const [message, setMessage] = useState("Search by mobile, Aadhaar, account number, or customer code.");
  const [isSearching, setIsSearching] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [formMode, setFormMode] = useState<FormMode>("create");
  const [editingAccountId, setEditingAccountId] = useState<number | null>(null);
  const [form, setForm] = useState<CustomerFormState>(emptyForm);
  const [formError, setFormError] = useState<string | null>(null);
  const [transactionForm, setTransactionForm] = useState<TransactionFormState>(emptyTransactionForm);
  const [transactionError, setTransactionError] = useState<string | null>(null);
  const [lastTransaction, setLastTransaction] = useState<Transaction | null>(null);
  const [dailySummary, setDailySummary] = useState<DailySummary | null>(null);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [reportDate, setReportDate] = useState(today);
  const [dailyReport, setDailyReport] = useState<DailyReport | null>(null);
  const [isLoadingList, setIsLoadingList] = useState(false);
  const [licenseStatus, setLicenseStatus] = useState<LicenseStatus | null>(null);
  const [setupForm, setSetupForm] = useState({ clientName: "", clientMobile: "", adminPassword: "", licenseKey: "", defaultBankId: "" });
  const [setupMessage, setSetupMessage] = useState<string | null>(null);

  useEffect(() => {
    async function boot() {
      try {
        const [bankOptions, summary, license] = await Promise.all([
          invoke<BankOption[]>("list_banks"),
          invoke<DailySummary>("get_daily_summary", { input: { date: today } }),
          invoke<LicenseStatus>("get_license_status"),
        ]);
        setBanks(bankOptions);
        setDailySummary(summary);
        setLicenseStatus(license);
        setSetupForm((current) => ({ ...current, clientName: license.clientName ?? "", clientMobile: license.clientMobile ?? "", defaultBankId: String(license.defaultBankId ?? bankOptions[0]?.id ?? "") }));
        if (!license.isActivated) setPage("setup");
        setForm((current) => ({ ...current, bankId: current.bankId || String(license.defaultBankId ?? bankOptions[0]?.id ?? "") }));
      } catch (error) {
        setBanksError(error instanceof Error ? error.message : String(error));
      }
    }

    void boot();
  }, [today]);

  const primaryAccount = selectedCustomer?.accounts[0] ?? null;
  const summaryText = useMemo(() => {
    if (selectedCustomer) return `${selectedCustomer.customerCode} selected`;
    if (searchResults.length > 1) return `${searchResults.length} matching customers found`;
    return message;
  }, [message, searchResults.length, selectedCustomer]);

  async function refreshToday() {
    const [summary, ledger] = await Promise.all([
      invoke<DailySummary>("get_daily_summary", { input: { date: today } }),
      invoke<Transaction[]>("get_daily_transactions", { input: { date: today } }),
    ]);
    setDailySummary(summary);
    setTransactions(ledger);
  }

  async function handleSearch() {
    const query = searchQuery.trim();
    setFormError(null);
    setTransactionError(null);
    setSelectedCustomer(null);
    setSearchResults([]);

    if (query.length < 4) {
      setMessage("Enter at least 4 characters to search.");
      return;
    }

    setIsSearching(true);
    try {
      const results = await invoke<Customer[]>("search_customer", { input: { query } });
      setSearchResults(results);
      if (results.length === 1) {
        setSelectedCustomer(results[0]);
        setMessage("Customer loaded. Enter a deposit or withdrawal below.");
      } else if (results.length === 0) {
        setMessage("Customer not found. Use Add Customer to create a profile.");
      } else {
        setMessage("Select the correct customer from the results.");
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSearching(false);
    }
  }

  function startCreate(seed = "") {
    const digits = seed.replace(/\D/g, "");
    setFormMode("create");
    setEditingAccountId(null);
    setForm({
      ...emptyForm,
      mobile: digits.length >= 10 ? digits.slice(-10) : "",
      accountNumber: /[a-zA-Z]/.test(seed) || digits.length > 10 ? seed : "",
      bankId: String(licenseStatus?.defaultBankId ?? banks[0]?.id ?? ""),
    });
    setFormError(null);
    setPage("customer-form");
  }

  function startEdit(customer: Customer) {
    const account = customer.accounts[0];
    setSelectedCustomer(customer);
    setFormMode("edit");
    setEditingAccountId(account?.id ?? null);
    setForm({
      fullName: customer.fullName,
      mobile: customer.mobileDisplay ?? "",
      aadhaar: "",
      bankId: String(account?.bankId ?? banks[0]?.id ?? ""),
      accountNumber: "",
      addressLine: customer.addressLine ?? "",
      city: customer.city ?? "",
    });
    setFormError("For edit, re-enter account number and Aadhaar only if they should be stored now. Full sensitive values are never shown back from storage.");
    setPage("customer-form");
  }

  async function handleSaveCustomer(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setFormError(null);

    const parsed = customerSchema.safeParse({ ...form, bankId: form.bankId ? Number(form.bankId) : 0 });
    if (!parsed.success) {
      setFormError(parsed.error.issues[0]?.message ?? "Check customer details.");
      return;
    }

    setIsSaving(true);
    try {
      const payload = toCustomerInput(parsed.data);
      const customer =
        formMode === "edit" && selectedCustomer && editingAccountId
          ? await invoke<Customer>("update_customer", {
              input: { ...payload, customerId: selectedCustomer.id, accountId: editingAccountId } satisfies UpdateCustomerInput,
              userId: user.id,
            })
          : await invoke<Customer>("create_customer", { input: payload, userId: user.id });

      setSelectedCustomer(customer);
      setSearchResults([customer]);
      setSearchQuery(customer.mobileDisplay ?? customer.customerCode);
      setMessage(formMode === "edit" ? "Customer updated." : "Customer created. You can now record a transaction.");
      setForm(emptyFormWithBank(banks));
      setFormMode("create");
      setEditingAccountId(null);
      setPage("search");
    } catch (error) {
      setFormError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function handleCreateTransaction(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setTransactionError(null);
    setLastTransaction(null);

    if (!selectedCustomer || !primaryAccount) {
      setTransactionError("Search and select a customer before saving a transaction.");
      return;
    }

    const parsed = transactionSchema.safeParse(transactionForm);
    if (!parsed.success) {
      setTransactionError(parsed.error.issues[0]?.message ?? "Check transaction details.");
      return;
    }

    const amountPaise = amountToPaise(parsed.data.amount);
    if (amountPaise <= 0) {
      setTransactionError("Amount must be greater than zero.");
      return;
    }

    setIsSaving(true);
    try {
      const transaction = await invoke<Transaction>("create_transaction", {
        input: {
          customerId: selectedCustomer.id,
          bankAccountId: primaryAccount.id,
          transactionType: parsed.data.transactionType,
          amountPaise,
          remarks: parsed.data.remarks,
          createdBy: user.id,
        },
      });
      setLastTransaction(transaction);
      setTransactionForm(emptyTransactionForm);
      setMessage(`${transaction.transactionNumber} saved.`);
      await refreshToday();
    } catch (error) {
      setTransactionError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }


  async function handleCompleteSetup(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSetupMessage(null);
    setIsSaving(true);
    try {
      const status = await invoke<LicenseStatus>("complete_client_setup", {
        input: { ...setupForm, defaultBankId: setupForm.defaultBankId ? Number(setupForm.defaultBankId) : null },
      });
      setLicenseStatus(status);
      setSetupMessage("Setup completed and license activated.");
      if (status.isActivated) setPage("search");
    } catch (error) {
      setSetupMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function openCustomers() {
    setPage("customers");
    setIsLoadingList(true);
    try {
      setCustomers(await invoke<Customer[]>("list_customers"));
    } finally {
      setIsLoadingList(false);
    }
  }

  async function openCashManagement() {
    setPage("cash-management");
    setIsLoadingList(true);
    try {
      await refreshToday();
    } finally {
      setIsLoadingList(false);
    }
  }

  async function loadReport(date = reportDate) {
    setIsLoadingList(true);
    try {
      setDailyReport(await invoke<DailyReport>("get_daily_report", { input: { date } }));
    } finally {
      setIsLoadingList(false);
    }
  }

  async function openReport() {
    setPage("report");
    await loadReport(reportDate);
  }

  return (
    <main className="app-shell">
      <header className="top-bar">
        <div className="header-brand">
          <p className="eyebrow">{settings.shopName}</p>
          <h1>Cash Ledger</h1>
        </div>

        <nav className="page-tabs" aria-label="Cash Ledger pages">
          <button type="button" className={page === "search" ? "active-tab" : ""} onClick={() => setPage("search")}>Search</button>
          <button type="button" className={page === "customers" ? "active-tab" : ""} onClick={openCustomers}>Customers</button>
          <button type="button" className={page === "cash-management" ? "active-tab" : ""} onClick={openCashManagement}>Cash Management</button>
          <button type="button" className={page === "report" ? "active-tab" : ""} onClick={openReport}>End Day Report</button>
          <button type="button" className="header-add-button" onClick={() => startCreate(searchQuery)}>+ Add Customer</button>
        </nav>

        <div className="operator-block">
          <span>{appDate}</span>
          <strong>{user.fullName || user.username}</strong>
          <button type="button" onClick={onLogout}>Logout</button>
        </div>
      </header>

      {page === "search" ? renderSearchPage() : null}
      {page === "customer-form" ? renderCustomerFormPage() : null}
      {page === "customers" ? renderCustomersPage() : null}
      {page === "cash-management" ? renderCashManagementPage() : null}
      {page === "report" ? renderReportPage() : null}
      {page === "setup" ? renderSetupPage() : null}
    </main>
  );


  function renderSetupPage() {
    return (
      <section className="page-shell narrow-page">
        <form className="work-surface customer-form" onSubmit={handleCompleteSetup}>
          <div className="section-header">
            <h2>Client Setup & Activation</h2>
            <span className={licenseStatus?.isActivated ? "pill" : "muted-text"}>
              {licenseStatus?.isActivated ? "Activated" : "Not Activated"}
            </span>
          </div>

          <div className="setup-code-box">
            <span>Machine fingerprint code</span>
            <strong>{licenseStatus?.machineCode ?? "Loading..."}</strong>
            <small>Client sends this code to you. You generate the license key for this machine.</small>
          </div>

          <div className="form-two-column">
            <label>Client name<input value={setupForm.clientName} onChange={(event) => setSetupForm((current) => ({ ...current, clientName: event.target.value }))} /></label>
            <label>Mobile number<input value={setupForm.clientMobile} onChange={(event) => setSetupForm((current) => ({ ...current, clientMobile: event.target.value }))} /></label>
            <label>New admin password<input type="password" value={setupForm.adminPassword} onChange={(event) => setSetupForm((current) => ({ ...current, adminPassword: event.target.value }))} /></label>
            <label>License key<input value={setupForm.licenseKey} onChange={(event) => setSetupForm((current) => ({ ...current, licenseKey: event.target.value.toUpperCase() }))} placeholder="XXXX-XXXX-XXXX-XXXX" /></label>
            <label>Default bank<select value={setupForm.defaultBankId} onChange={(event) => setSetupForm((current) => ({ ...current, defaultBankId: event.target.value }))}><option value="">Select bank</option>{banks.map((bank) => <option key={bank.id} value={bank.id}>{bank.name}</option>)}</select></label>
          </div>

          {setupMessage ? <p className={licenseStatus?.isActivated ? "success-note" : "form-error"}>{setupMessage}</p> : null}
          <button type="submit" disabled={isSaving}>{isSaving ? "Saving..." : "Activate Setup"}</button>
        </form>
      </section>
    );
  }

  function renderSearchPage() {
    return (
      <section className="page-shell">
        <div className="work-surface search-surface">
          <div className="search-row">
            <label className="search-label">
              Search Customer
              <input
                className="search-input"
                placeholder="Mobile / Aadhaar / Account Number"
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") void handleSearch();
                }}
              />
            </label>
            <button type="button" className="secondary-button" onClick={() => { setSearchQuery(""); setSelectedCustomer(null); setSearchResults([]); setMessage("Search by mobile, Aadhaar, account number, or customer code."); }}>Clear</button>
            <button type="button" onClick={handleSearch} disabled={isSearching}>{isSearching ? "Searching..." : "Search"}</button>
          </div>
          <p className="muted-text">{summaryText}</p>

          {searchResults.length > 1 ? <CustomerResults customers={searchResults} onSelect={setSelectedCustomer} /> : null}
          {selectedCustomer ? renderSelectedCustomer() : <div className="empty-state">Search a customer to record cash deposit or cash withdrawal.</div>}
        </div>
      </section>
    );
  }

  function renderSelectedCustomer() {
    return (
      <div className="single-column-stack">
        <section className="customer-panel" aria-label="Customer details">
          <div className="section-header">
            <h2>{selectedCustomer?.fullName}</h2>
            {selectedCustomer ? <button type="button" className="secondary-button" onClick={() => startEdit(selectedCustomer)}>Edit Customer</button> : null}
          </div>
          {selectedCustomer ? <CustomerDetails customer={selectedCustomer} /> : null}
        </section>

        <form className="transaction-panel" onSubmit={handleCreateTransaction}>
          <div className="section-header"><h2>Cash Transaction</h2></div>
          <div className="transaction-controls">
            <label className="radio-card"><input type="radio" checked={transactionForm.transactionType === "DEPOSIT"} onChange={() => setTransactionForm((current) => ({ ...current, transactionType: "DEPOSIT" }))} /> Cash Deposit</label>
            <label className="radio-card"><input type="radio" checked={transactionForm.transactionType === "WITHDRAWAL"} onChange={() => setTransactionForm((current) => ({ ...current, transactionType: "WITHDRAWAL" }))} /> Cash Withdrawal</label>
            <label>Amount<input inputMode="decimal" value={transactionForm.amount} onChange={(event) => setTransactionForm((current) => ({ ...current, amount: event.target.value }))} placeholder="0.00" /></label>
            <label>Remarks<input value={transactionForm.remarks} onChange={(event) => setTransactionForm((current) => ({ ...current, remarks: event.target.value }))} /></label>
          </div>
          {transactionError ? <p className="form-error">{transactionError}</p> : null}
          {lastTransaction ? <p className="success-note">Saved {lastTransaction.transactionNumber} for {formatCurrency(lastTransaction.amountPaise)}.</p> : null}
          <button type="submit" disabled={isSaving}>{isSaving ? "Saving..." : "Save Transaction"}</button>
        </form>
      </div>
    );
  }

  function renderCustomerFormPage() {
    return (
      <section className="page-shell narrow-page">
        <form className="work-surface customer-form" onSubmit={handleSaveCustomer}>
          <div className="section-header">
            <h2>{formMode === "edit" ? "Edit Customer" : "Add Customer"}</h2>
            <button type="button" className="secondary-button" onClick={() => setPage("search")}>Back to Search</button>
          </div>
          <div className="form-two-column">
            <label>Customer name<input value={form.fullName} onChange={updateField("fullName")} /></label>
            <label>Mobile number<input value={form.mobile} onChange={updateField("mobile")} /></label>
            <div className="address-city-row span-two"><label>Address<input value={form.addressLine} onChange={updateField("addressLine")} /></label><label>City / village<input value={form.city} onChange={updateField("city")} /></label></div>
            <label className="span-two">Aadhaar optional<input value={form.aadhaar} onChange={updateField("aadhaar")} placeholder="Masked after save" /></label>
            <label>Bank<select value={form.bankId} onChange={updateField("bankId")}><option value="">Select bank</option>{banks.map((bank) => <option key={bank.id} value={bank.id}>{bank.name}</option>)}</select></label>
            <label>Account number<input value={form.accountNumber} onChange={updateField("accountNumber")} /></label>
          </div>
          {banksError ? <p className="form-error">{banksError}</p> : null}
          {formError ? <p className="form-error">{formError}</p> : null}
          <button type="submit" disabled={isSaving || banks.length === 0}>{isSaving ? "Saving..." : formMode === "edit" ? "Save Customer" : "Add Customer"}</button>
        </form>
      </section>
    );
  }

  function renderCustomersPage() {
    return (
      <section className="page-shell wide-page">
        <div className="work-surface">
          <div className="section-header"><h2>Customer Listing</h2><button type="button" onClick={openCustomers}>Refresh</button></div>
          {isLoadingList ? <p className="muted-text">Loading customers...</p> : <CustomerTable customers={customers} onEdit={startEdit} />}
        </div>
      </section>
    );
  }

  function renderCashManagementPage() {
    return (
      <section className="page-shell wide-page">
        <div className="work-surface">
          <div className="section-header"><h2>Daily Cash Management</h2><button type="button" onClick={openCashManagement}>Refresh</button></div>
          {dailySummary ? <SummaryCards summary={dailySummary} /> : null}
          {isLoadingList ? <p className="muted-text">Loading cash ledger...</p> : <TransactionTable transactions={transactions} />}
        </div>
      </section>
    );
  }

  function renderReportPage() {
    return (
      <section className="page-shell wide-page">
        <div className="work-surface">
          <div className="section-header"><h2>End Day Report</h2><button type="button" onClick={printEndDayReport}>Print</button></div>
          <div className="report-toolbar"><label>Report date<input type="date" value={reportDate} onChange={(event) => setReportDate(event.target.value)} /></label><button type="button" onClick={() => loadReport(reportDate)}>Load Report</button></div>
          {dailyReport ? <><p className="report-period">Reporting interval: {formatReportInterval(reportDate)}</p><SummaryCards summary={dailyReport.summary} /><EndDayTransactionTable transactions={dailyReport.transactions} /></> : <div className="empty-state">Load a report date to view end-of-day totals.</div>}
        </div>
      </section>
    );
  }

  function printEndDayReport() {
    const originalTitle = document.title;
    const restoreTitle = () => {
      document.title = originalTitle;
      window.removeEventListener("afterprint", restoreTitle);
    };

    document.title = `End-Day-Report-${formatPrintFileTimestamp(new Date())}`;
    window.addEventListener("afterprint", restoreTitle);
    window.print();
  }

  function updateField(field: keyof CustomerFormState) {
    return (event: ChangeEvent<HTMLInputElement | HTMLSelectElement>) => setForm((current) => ({ ...current, [field]: event.target.value }));
  }
}

function CustomerResults({ customers, onSelect }: { customers: Customer[]; onSelect: (customer: Customer) => void }) {
  return <div className="result-list">{customers.map((customer) => <button type="button" className="result-row" key={customer.id} onClick={() => onSelect(customer)}><strong>{customer.fullName}</strong><span>{customer.mobileDisplay ?? "No mobile"}</span><span>{customer.accounts[0]?.bankName ?? "No bank"}</span><span>{customer.accounts[0]?.accountDisplay ?? "No account"}</span></button>)}</div>;
}

function CustomerDetails({ customer }: { customer: Customer }) {
  const account = customer.accounts[0];
  return <dl className="detail-grid"><div><dt>Code</dt><dd>{customer.customerCode}</dd></div><div><dt>Mobile</dt><dd>{customer.mobileDisplay ?? "-"}</dd></div><div><dt>Aadhaar</dt><dd>{customer.aadhaarMasked ?? "Not stored"}</dd></div><div><dt>Bank</dt><dd>{account?.bankName ?? "-"}</dd></div><div><dt>Account</dt><dd>{account?.accountDisplay ?? "-"}</dd></div><div><dt>Address</dt><dd>{formatAddress(customer)}</dd></div></dl>;
}

function CustomerTable({ customers, onEdit }: { customers: Customer[]; onEdit: (customer: Customer) => void }) {
  if (customers.length === 0) return <div className="empty-state">No customers found.</div>;
  return <div className="data-table"><div className="table-header customer-table-grid"><span>Customer</span><span>Mobile</span><span>Aadhaar</span><span>Bank</span><span>Account</span><span>Address</span><span></span></div>{customers.map((customer) => <div className="table-row customer-table-grid" key={customer.id}><strong>{customer.fullName}<small>{customer.customerCode}</small></strong><span>{customer.mobileDisplay ?? "-"}</span><span>{customer.aadhaarDisplay ?? "-"}</span><span>{customer.accounts[0]?.bankName ?? "-"}</span><span>{customer.accounts[0]?.accountDisplay ?? "-"}</span><span>{formatAddress(customer)}</span><button type="button" className="secondary-button" onClick={() => onEdit(customer)}>Edit</button></div>)}</div>;
}

function SummaryCards({ summary }: { summary: DailySummary }) {
  return <div className="summary-grid"><div><span>Deposits</span><strong>{formatCurrency(summary.depositTotalPaise)}</strong><small>{summary.depositCount} entries</small></div><div><span>Withdrawals</span><strong>{formatCurrency(summary.withdrawalTotalPaise)}</strong><small>{summary.withdrawalCount} entries</small></div><div><span>Net Movement</span><strong>{formatCurrency(summary.netMovementPaise)}</strong><small>Deposits - withdrawals</small></div><div><span>Customers</span><strong>{summary.uniqueCustomers}</strong><small>{summary.transactionCount} transactions</small></div></div>;
}

function TransactionTable({ transactions }: { transactions: Transaction[] }) {
  if (transactions.length === 0) return <div className="empty-state">No cash transactions found for this day.</div>;
  return <div className="data-table"><div className="table-header transaction-table-grid"><span>Date &amp; Time</span><span>Transaction</span><span>Customer</span><span>Aadhaar</span><span>Bank</span><span>Type</span><span>Amount</span></div>{transactions.map((transaction) => <div className="table-row transaction-table-grid" key={transaction.id}><span>{formatDateTime(transaction.transactionTimestamp)}</span><strong>{transaction.transactionNumber}<small>{transaction.remarks ?? ""}</small></strong><span>{transaction.customerName}<small>{transaction.mobileDisplay ?? ""}</small></span><span>{transaction.aadhaarDisplay ?? "-"}</span><span>{transaction.bankName}<small>{transaction.accountDisplay}</small></span><span className={transaction.transactionType === "DEPOSIT" ? "type-deposit" : "type-withdrawal"}>{transaction.transactionType === "DEPOSIT" ? "Deposit" : "Withdrawal"}</span><strong>{formatCurrency(transaction.amountPaise)}</strong></div>)}</div>;
}

function EndDayTransactionTable({ transactions }: { transactions: Transaction[] }) {
  if (transactions.length === 0) return <div className="empty-state">No cash transactions found for this day.</div>;
  return <div className="data-table end-day-transaction-table"><div className="table-header end-day-transaction-table-grid"><span>Date &amp; Time</span><span>Customer</span><span>Mobile</span><span>Aadhaar</span><span>Bank</span><span>Account</span><span>Type</span><span>Amount</span></div>{transactions.map((transaction) => <div className="table-row end-day-transaction-table-grid" key={transaction.id}><span>{formatDateTime(transaction.transactionTimestamp)}</span><span>{transaction.customerName}</span><span>{transaction.mobileDisplay ?? "-"}</span><span>{transaction.aadhaarDisplay ?? "-"}</span><span>{shortenBankName(transaction.bankName)}</span><span>{transaction.accountDisplay ?? "-"}</span><span className={transaction.transactionType === "DEPOSIT" ? "type-deposit" : "type-withdrawal"}>{transaction.transactionType === "DEPOSIT" ? "Deposit" : "Withdrawal"}</span><strong>{formatCurrency(transaction.amountPaise)}</strong></div>)}</div>;
}

function toCustomerInput(values: CustomerFormValues): CustomerInput {
  return { fullName: values.fullName, mobile: values.mobile, aadhaar: values.aadhaar, bankId: values.bankId, accountNumber: values.accountNumber, addressLine: values.addressLine, city: values.city };
}

function emptyFormWithBank(banks: BankOption[]): CustomerFormState {
  return { ...emptyForm, bankId: String(banks[0]?.id ?? "") };
}

function getLocalDateInputValue(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatReportInterval(dateValue: string) {
  const formattedDate = formatDateForReport(dateValue);
  return `${formattedDate}, 12:00 AM – ${formattedDate}, 11:59 PM`;
}

function formatDateForReport(dateValue: string) {
  const [year, month, day] = dateValue.split("-");
  if (!year || !month || !day) return dateValue;
  return `${day}/${month}/${year}`;
}

function formatPrintFileTimestamp(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");
  return `${year}-${month}-${day}_${hours}-${minutes}-${seconds}`;
}

function shortenBankName(bankName: string) {
  const normalized = bankName.trim().toLowerCase();
  if (normalized === "punjab national bank") return "PNB";
  if (normalized === "state bank of india" || normalized === "state bank india") return "SBI";
  return bankName;
}

function formatAddress(customer: Customer) {
  const parts = [customer.addressLine, customer.city].filter(Boolean);
  return parts.length ? parts.join(", ") : "-";
}

function formatCurrency(paise: number) {
  return new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" }).format(paise / 100);
}

function formatDateTime(timestamp: string) {
  const [date, time] = timestamp.split(" ");
  const [year, month, day] = date?.split("-") ?? [];
  const formattedDate = year && month && day ? `${day}/${month}/${year}` : date;
  return time ? `${formattedDate} ${time.slice(0, 5)}` : timestamp;
}
