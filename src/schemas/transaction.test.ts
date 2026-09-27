import { describe, expect, it } from "vitest";
import { amountToPaise, transactionSchema } from "./transaction";

describe("transactionSchema", () => {
  it("accepts a valid deposit amount", () => {
    expect(transactionSchema.safeParse({ transactionType: "DEPOSIT", amount: "1250.50", remarks: "" }).success).toBe(true);
  });

  it("converts rupees to paise", () => {
    expect(amountToPaise("1250.50")).toBe(125050);
    expect(amountToPaise("10")).toBe(1000);
  });
});
