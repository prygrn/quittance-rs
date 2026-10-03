import { describe, expectTypeOf, it } from "vitest";

import type { CommandErrorCode, ReceiptInput, TemplateInfo } from "./api";

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

  it("lists the error codes the Tauri commands may return", () => {
    // Arrange
    type ExpectedCommandErrorCode =
      "validation" | "template" | "signature" | "pdf" | "mail" | "config" | "unknown";

    // Act & Assert
    expectTypeOf<CommandErrorCode>().toEqualTypeOf<ExpectedCommandErrorCode>();
  });
});
