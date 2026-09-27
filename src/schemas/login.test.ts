import { describe, expect, it } from "vitest";
import { loginSchema } from "./login";

describe("loginSchema", () => {
  it("accepts a username and password", () => {
    const result = loginSchema.safeParse({
      username: "admin",
      password: "admin123",
    });

    expect(result.success).toBe(true);
  });

  it("rejects an empty password", () => {
    const result = loginSchema.safeParse({
      username: "admin",
      password: "",
    });

    expect(result.success).toBe(false);
  });
});
