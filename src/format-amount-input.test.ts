import { describe, expect, it } from "vitest";

import { parseAmountCents } from "./amount";
import { formatAmountInput } from "./format-amount-input";

describe("formatAmountInput", () => {
  it.each<[number, string]>([
    [65_000, "650,00"],
    [5_050, "50,50"],
    [5, "0,05"],
    [0, "0,00"],
    [123_456, "1234,56"],
    [Number.MAX_SAFE_INTEGER, "90071992547409,91"],
  ])("writes %d cents as %j", (cents, expected) => {
    // Arrange & Act
    const text = formatAmountInput(cents);

    // Assert
    expect(text).toBe(expected);
  });

  it("writes an amount that the form reads back unchanged", () => {
    // Arrange
    const cents = 123_456_789;

    // Act
    const parsed = parseAmountCents(formatAmountInput(cents));

    // Assert
    expect(parsed).toEqual({ isValid: true, cents });
  });

  it.each([-1, 1.5, Number.NaN, Number.MAX_SAFE_INTEGER + 1])(
    "rejects %d, which is not a count of cents",
    (cents: number) => {
      // Arrange & Act
      const formatting = (): string => formatAmountInput(cents);

      // Assert
      expect(formatting).toThrow("[UI_INVALID_AMOUNT_CENTS]");
    },
  );
});
