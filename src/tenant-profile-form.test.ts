import { describe, expect, it } from "vitest";

import type { FieldErrorCode } from "./receipt-form";
import {
  type TenantProfileFields,
  type TenantProfileFormField,
  type TenantProfileFormValues,
  validateTenantProfileForm,
} from "./tenant-profile-form";

const ALL_FIELDS: readonly TenantProfileFormField[] = [
  "tenantName",
  "tenantAddress",
  "tenantEmail",
  "propertyAddress",
  "rentAmount",
  "chargesAmount",
];

function sampleValues(): TenantProfileFormValues {
  return {
    tenantName: "Jeanne Martin",
    tenantAddress: "8 rue Victor Hugo, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    rentAmount: "650",
    chargesAmount: "50,50",
  };
}

function sampleFields(): TenantProfileFields {
  return {
    tenantName: "Jeanne Martin",
    tenantAddress: "8 rue Victor Hugo, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    rentCents: 65_000,
    chargesCents: 5_050,
  };
}

function valuesWith(overrides: Partial<TenantProfileFormValues>): TenantProfileFormValues {
  return { ...sampleValues(), ...overrides };
}

describe("validateTenantProfileForm", () => {
  it("maps valid raw values to the content of a tenant profile", () => {
    // Arrange
    const values = sampleValues();

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({ isValid: true, fields: sampleFields() });
  });

  it("trims every field before mapping it", () => {
    // Arrange
    const values = valuesWith({
      tenantName: "  Jeanne Martin ",
      tenantAddress: "\t8 rue Victor Hugo, 75011 Paris\n",
      tenantEmail: " jeanne.martin@example.fr  ",
      propertyAddress: " 12 rue des Lilas, 75011 Paris ",
      rentAmount: " 650 ",
      chargesAmount: " 50,50",
    });

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({ isValid: true, fields: sampleFields() });
  });

  it.each(ALL_FIELDS)("reports a blank %s as missing", (field: TenantProfileFormField) => {
    // Arrange
    const values = valuesWith({ [field]: "  " });

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { [field]: "missing" } });
  });

  it("reports a malformed email", () => {
    // Arrange
    const values = valuesWith({ tenantEmail: "jeanne.martin@example" });

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { tenantEmail: "invalidEmail" } });
  });

  it.each<[TenantProfileFormField, string, FieldErrorCode]>([
    ["rentAmount", "six cent cinquante", "invalidAmount"],
    ["rentAmount", "-650", "negativeAmount"],
    ["chargesAmount", "50,505", "tooManyDecimals"],
    ["chargesAmount", "90071992547409,92", "amountTooLarge"],
  ])("reports %s %j as %s", (field, value, expectedError) => {
    // Arrange
    const values = valuesWith({ [field]: value });

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({ isValid: false, errors: { [field]: expectedError } });
  });

  it("reports every invalid field at once", () => {
    // Arrange
    const values = valuesWith({ tenantName: "", tenantEmail: "jeanne", rentAmount: "-1" });

    // Act
    const result = validateTenantProfileForm(values);

    // Assert
    expect(result).toEqual({
      isValid: false,
      errors: { tenantName: "missing", tenantEmail: "invalidEmail", rentAmount: "negativeAmount" },
    });
  });
});
