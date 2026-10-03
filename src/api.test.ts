import { describe, expectTypeOf, it } from "vitest";

import type { ReceiptInput, TemplateInfo } from "./api";

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
});
