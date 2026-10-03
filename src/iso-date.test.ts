import { describe, expect, it } from "vitest";

import { isValidIsoDate } from "./iso-date";

describe("isValidIsoDate", () => {
  it.each(["2026-10-01", "2026-12-31", "2024-02-29", "2000-02-29", "0001-01-01"])(
    "accepts %j",
    (value: string) => {
      // Arrange & Act
      const isValid = isValidIsoDate(value);

      // Assert
      expect(isValid).toBe(true);
    },
  );

  it.each([
    "",
    "2026-02-29",
    "1900-02-29",
    "2026-04-31",
    "2026-10-32",
    "2026-13-01",
    "2026-00-10",
    "2026-10-00",
    "2026-1-1",
    "01/10/2026",
    "01-10-2026",
    "2026-10-01T00:00",
    " 2026-10-01",
    "+02026-10-01",
    "aaaa-bb-cc",
  ])("rejects %j", (value: string) => {
    // Arrange & Act
    const isValid = isValidIsoDate(value);

    // Assert
    expect(isValid).toBe(false);
  });
});
