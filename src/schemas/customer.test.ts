import { describe, expect, it } from "vitest";
import { customerSchema } from "./customer";

describe("customerSchema", () => {
  it("accepts valid customer details", () => {
    const result = customerSchema.safeParse({
      fullName: "Ramesh Kumar",
      mobile: "9876543210",
      aadhaar: "1234 5678 9012",
      bankId: 1,
      accountNumber: "1234-567890",
      addressLine: "Biaora",
      city: "Biaora",
    });

    expect(result.success).toBe(true);
  });

  it("rejects invalid mobile numbers", () => {
    const result = customerSchema.safeParse({
      fullName: "Ramesh Kumar",
      mobile: "12345",
      aadhaar: "",
      bankId: 1,
      accountNumber: "1234567890",
      addressLine: "",
      city: "",
    });

    expect(result.success).toBe(false);
  });
});
