export type DbStatus = {
  databasePath: string;
  schemaVersion: number;
  userCount: number;
};

export type AppSettings = {
  shopName: string;
  ownerName: string | null;
  phone: string | null;
  address: string | null;
  automaticBackupEnabled: boolean;
  backupRetentionDays: number;
  autoLockMinutes: number;
};

export type AuthenticatedUser = {
  id: number;
  username: string;
  fullName: string | null;
  role: string;
};

export type BankOption = {
  id: number;
  name: string;
  shortName: string | null;
};

export type CustomerAccount = {
  id: number;
  bankId: number;
  bankName: string;
  accountLast4: string;
  accountMasked: string;
  accountDisplay: string;
  isPrimary: boolean;
};

export type Customer = {
  id: number;
  customerCode: string;
  fullName: string;
  mobileDisplay: string | null;
  mobileMasked: string | null;
  aadhaarDisplay: string | null;
  aadhaarMasked: string | null;
  addressLine: string | null;
  city: string | null;
  state: string | null;
  pinCode: string | null;
  accounts: CustomerAccount[];
};

export type CustomerInput = {
  fullName: string;
  mobile: string;
  aadhaar?: string | null;
  bankId: number;
  accountNumber: string;
  addressLine?: string | null;
  city?: string | null;
};

export type UpdateCustomerInput = CustomerInput & {
  customerId: number;
  accountId: number;
};

export type TransactionType = "DEPOSIT" | "WITHDRAWAL";

export type Transaction = {
  id: number;
  transactionNumber: string;
  customerId: number;
  customerName: string;
  mobileDisplay: string | null;
  mobileMasked: string | null;
  aadhaarDisplay: string | null;
  bankName: string;
  accountMasked: string;
  accountDisplay: string;
  transactionType: TransactionType;
  amountPaise: number;
  transactionTimestamp: string;
  remarks: string | null;
  status: string;
};

export type DailySummary = {
  date: string;
  depositTotalPaise: number;
  withdrawalTotalPaise: number;
  depositCount: number;
  withdrawalCount: number;
  transactionCount: number;
  uniqueCustomers: number;
  netMovementPaise: number;
};

export type DailyReport = {
  summary: DailySummary;
  transactions: Transaction[];
};

export type LicenseStatus = {
  machineCode: string;
  isActivated: boolean;
  clientName: string | null;
  clientMobile: string | null;
  defaultBankId: number | null;
};
