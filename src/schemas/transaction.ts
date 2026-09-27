import { z } from "zod";

export const transactionSchema = z.object({
  transactionType: z.enum(["DEPOSIT", "WITHDRAWAL"], {
    required_error: "Select deposit or withdrawal.",
  }),
  amount: z
    .string()
    .trim()
    .regex(/^\d+(?:\.\d{1,2})?$/, "Enter a valid amount with up to 2 decimals."),
  remarks: z.string().trim().optional().transform((value) => value || null),
});

export type TransactionFormValues = z.infer<typeof transactionSchema>;

export function amountToPaise(amount: string) {
  const [rupees, paise = ""] = amount.split(".");
  return Number(rupees) * 100 + Number(paise.padEnd(2, "0"));
}
