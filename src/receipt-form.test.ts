import { describe, expect, it } from "vitest";

import {
  type FieldErrorCode,
  type ReceiptFormField,
  type ReceiptFormValues,
  validateReceiptForm,
} from "./receipt-form";
import { sampleReceiptFormValues, sampleReceiptInput } from "./test-fixtures";

const ALL_FIELDS: ReceiptFormField[] = [
  "tenantName",
  "tenantAddress",
  "tenantEmail",
  "propertyAddress",
  "periodStart",
  "periodEnd",
  "rentAmount",
  "chargesAmount",
  "paymentDate",
];

const DATE_FIELDS: ReceiptFormField[] = ["periodStart", "periodEnd", "paymentDate"];

function formValuesWith(overrides: Partial<ReceiptFormValues>): ReceiptFormValues {
  return { ...sampleReceiptFormValues(), ...overrides };
}

describe("validateReceiptForm", () => {
  it("maps valid raw values to a receipt input", () => {
    // Arrange
    const values = sampleReceiptFormValues();

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: true, input: sampleReceiptInput() });
  });

  it("trims every field before mapping it", () => {
    // Arrange
    const values = formValuesWith({
      tenantName: "  Jeanne Martin ",
      tenantAddress: "\t12 rue des Lilas, 75011 Paris\n",
      tenantEmail: " jeanne.martin@example.fr  ",
      propertyAddress: " 12 rue des Lilas, 75011 Paris ",
      periodStart: " 2026-10-01 ",
      periodEnd: "2026-10-31\n",
      rentAmount: " 650 ",
      chargesAmount: " 50,50",
      paymentDate: "\t2026-10-05",
    });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: true, input: sampleReceiptInput() });
  });

  it.each(ALL_FIELDS)("reports %s as missing when it is blank", (field: ReceiptFormField) => {
    // Arrange
    const values = formValuesWith({ [field]: "   " });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { [field]: "missing" } });
  });

  it.each(["jeanne.martin", "jeanne@example", "jeanné@exemple.fr", "jeanne martin@example.fr"])(
    "reports %j as an invalid tenant email",
    (email: string) => {
      // Arrange
      const values = formValuesWith({ tenantEmail: email });

      // Act
      const result = validateReceiptForm(values);

      // Assert
      expect(result).toEqual({ isValid: false, errors: { tenantEmail: "invalidEmail" } });
    },
  );

  it.each(DATE_FIELDS)("reports %s as an invalid date when it does not exist", (field) => {
    // Arrange
    const values = formValuesWith({ [field]: "2026-02-30" });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { [field]: "invalidDate" } });
  });

  it("reports the period end when the period ends before it starts", () => {
    // Arrange
    const values = formValuesWith({ periodStart: "2026-10-31", periodEnd: "2026-10-01" });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { periodEnd: "invertedPeriod" } });
  });

  it("accepts a one day period", () => {
    // Arrange
    const values = formValuesWith({ periodStart: "2026-10-15", periodEnd: "2026-10-15" });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result.isValid).toBe(true);
  });

  it("compares period bounds across years", () => {
    // Arrange
    const values = formValuesWith({ periodStart: "2026-12-15", periodEnd: "2027-01-14" });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result.isValid).toBe(true);
  });

  it("does not report an inverted period when a bound is invalid", () => {
    // Arrange
    const values = formValuesWith({ periodStart: "2026-13-01", periodEnd: "2026-10-01" });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { periodStart: "invalidDate" } });
  });

  it.each<[ReceiptFormField, string, FieldErrorCode]>([
    ["rentAmount", "abc", "invalidAmount"],
    ["rentAmount", "-650", "negativeAmount"],
    ["rentAmount", "650,505", "tooManyDecimals"],
    ["rentAmount", "90071992547409,92", "amountTooLarge"],
    ["chargesAmount", "1.234,56", "invalidAmount"],
    ["chargesAmount", "-5", "negativeAmount"],
    ["chargesAmount", "0,001", "tooManyDecimals"],
    ["chargesAmount", "100000000000000000", "amountTooLarge"],
  ])("reports %s %j as %s", (field, rawAmount, expectedError) => {
    // Arrange
    const values = formValuesWith({ [field]: rawAmount });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { [field]: expectedError } });
  });

  it("reports every invalid field at once", () => {
    // Arrange
    const values = formValuesWith({
      tenantName: "",
      tenantEmail: "jeanne",
      periodEnd: "2026-09-30",
      rentAmount: "-1",
      paymentDate: "05/10/2026",
    });

    // Act
    const result = validateReceiptForm(values);

    // Assert
    expect(result).toEqual({
      isValid: false,
      errors: {
        tenantName: "missing",
        tenantEmail: "invalidEmail",
        periodEnd: "invertedPeriod",
        rentAmount: "negativeAmount",
        paymentDate: "invalidDate",
      },
    });
  });
});
