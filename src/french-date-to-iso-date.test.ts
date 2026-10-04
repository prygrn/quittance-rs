import { describe, expect, it } from "vitest";

import { frenchDateToIsoDate } from "./french-date-to-iso-date";

describe("frenchDateToIsoDate", () => {
  it.each([
    ["01/09/2026", "2026-09-01"],
    ["1/9/2026", "2026-09-01"],
    ["31/12/2026", "2026-12-31"],
    [" 03/09/2026 ", "2026-09-03"],
    // L'existence de la date est vérifiée ensuite par `validateReceiptForm`.
    ["31/09/2026", "2026-09-31"],
  ])("converts %j to %j", (value: string, expected: string) => {
    // Arrange & Act
    const result = frenchDateToIsoDate(value);

    // Assert
    expect(result).toBe(expected);
  });

  it.each(["", "   ", "2026-09-01", "01-09-2026", "01/09/26", "001/09/2026", "le 1er septembre"])(
    "returns %j unchanged when it is not a dd/mm/yyyy date",
    (value: string) => {
      // Arrange & Act
      const result = frenchDateToIsoDate(value);

      // Assert
      expect(result).toBe(value);
    },
  );
});
