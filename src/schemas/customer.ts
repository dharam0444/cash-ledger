import { z } from "zod";

const optionalTrimmed = z
  .string()
  .trim()
  .optional()
  .transform((value) => (value ? value : null));

export const customerSchema = z.object({
  fullName: z
    .string()
    .trim()
    .min(2, "Customer name must be at least 2 characters.")
    .max(120, "Customer name is too long."),
  mobile: z
    .string()
    .trim()
    .regex(/^(?:\+?91)?[6-9][0-9]{9}$/, "Enter a valid Indian mobile number."),
  aadhaar: z
    .string()
    .trim()
    .optional()
    .transform((value) => (value ? value.replace(/\D/g, "") : null))
    .refine((value) => !value || value.length === 12, "Aadhaar must contain 12 digits."),
  bankId: z.coerce.number().int().positive("Select a bank."),
  accountNumber: z
    .string()
    .trim()
    .regex(/^[a-zA-Z0-9\s-]{6,40}$/, "Enter a valid account number."),
  addressLine: optionalTrimmed,
  city: optionalTrimmed
});

export type CustomerFormValues = z.infer<typeof customerSchema>;
