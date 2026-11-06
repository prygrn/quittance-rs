import { describe, expectTypeOf, it } from "vitest";

import type {
  CommandErrorCode,
  ReceiptInput,
  TemplateInfo,
  TenantProfileEntry,
  TenantProfilePayload,
} from "./api";

describe("api types", () => {
  it("mirrors the backend receipt input with ISO dates and integer cents", () => {
    // Arrange
    type ExpectedReceiptInput = {
      tenantName: string;
      tenantAddress: string;
      tenantEmail: string;
      propertyAddress: string;
      periodStart: string;
      periodEnd: string;
      rentCents: number;
      chargesCents: number;
      paymentDate: string;
    };

    // Act & Assert
    expectTypeOf<ReceiptInput>().toEqualTypeOf<ExpectedReceiptInput>();
  });

  it("describes a template by its id and label", () => {
    // Arrange
    type ExpectedTemplateInfo = { id: string; label: string };

    // Act & Assert
    expectTypeOf<TemplateInfo>().toEqualTypeOf<ExpectedTemplateInfo>();
  });

  it("mirrors the backend tenant profile payload with integer cents", () => {
    // Arrange
    type ExpectedTenantProfilePayload = {
      templateId: string;
      tenantName: string;
      tenantAddress: string;
      tenantEmail: string;
      propertyAddress: string;
      rentCents: number;
      chargesCents: number;
    };

    // Act & Assert
    expectTypeOf<TenantProfilePayload>().toEqualTypeOf<ExpectedTenantProfilePayload>();
  });

  it("describes a saved tenant profile by its id and label", () => {
    // Arrange
    type ExpectedTenantProfileEntry = { id: string; label: string };

    // Act & Assert
    expectTypeOf<TenantProfileEntry>().toEqualTypeOf<ExpectedTenantProfileEntry>();
  });

  it("lists the error codes the Tauri commands may return", () => {
    // Arrange
    type ExpectedCommandErrorCode =
      | "validation"
      | "template"
      | "signature"
      | "pdf"
      | "mail"
      | "config"
      | "tenantProfile"
      | "tenantProfileExists"
      | "unknown";

    // Act & Assert
    expectTypeOf<CommandErrorCode>().toEqualTypeOf<ExpectedCommandErrorCode>();
  });
});
