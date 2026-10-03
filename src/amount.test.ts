import { describe, expect, it } from "vitest";

import { type AmountErrorCode, parseAmountCents } from "./amount";

describe("parseAmountCents", () => {
  it.each([
    ["650", 65_000],
    ["650,5", 65_050],
    ["650,50", 65_050],
    ["1 234,56", 123_456],
    ["1234.56", 123_456],
    ["0", 0],
    ["0,05", 5],
    ["007", 700],
    ["  650,50  ", 65_050],
    ["12 345 678,9", 1_234_567_890],
    ["1 234,56", 123_456],
    ["1 234,56", 123_456],
    ["90 071 992 547 409,91", Number.MAX_SAFE_INTEGER],
  ])("converts %j to %d cents", (rawAmount: string, expectedCents: number) => {
    // Arrange & Act
    const result = parseAmountCents(rawAmount);

    // Assert
    expect(result).toEqual({ isValid: true, cents: expectedCents });
  });

  it.each<[string, AmountErrorCode]>([
    ["", "invalidAmount"],
    ["   ", "invalidAmount"],
    ["abc", "invalidAmount"],
    ["12a", "invalidAmount"],
    ["650,", "invalidAmount"],
    [",50", "invalidAmount"],
    ["1.234,56", "invalidAmount"],
    ["1,234,56", "invalidAmount"],
    ["12 34", "invalidAmount"],
    ["1  234", "invalidAmount"],
    ["+650", "invalidAmount"],
    ["650 €", "invalidAmount"],
    ["1e3", "invalidAmount"],
    ["Infinity", "invalidAmount"],
    ["0x10", "invalidAmount"],
    ["-650", "negativeAmount"],
    ["-0,50", "negativeAmount"],
    [" -1", "negativeAmount"],
    ["650,505", "tooManyDecimals"],
    ["0.001", "tooManyDecimals"],
    ["90071992547409,92", "amountTooLarge"],
    ["100000000000000000", "amountTooLarge"],
  ])("rejects %j as %s", (rawAmount: string, expectedError: AmountErrorCode) => {
    // Arrange & Act
    const result = parseAmountCents(rawAmount);

    // Assert
    expect(result).toEqual({ isValid: false, error: expectedError });
  });
});
