import { describe, expect, it } from "vitest";

import { formatEuroCents } from "./format-euro-cents";

// Intl sépare milliers et symbole par des espaces insécables : on les normalise.
function withPlainSpaces(text: string): string {
  return text.replace(/\s/g, " ");
}

describe("formatEuroCents", () => {
  it.each([
    [73_000, "730,00 €"],
    [123_456, "1 234,56 €"],
    [5, "0,05 €"],
    [0, "0,00 €"],
  ])("formats %d cents as %j", (cents: number, expected: string) => {
    // Arrange & Act
    const result = formatEuroCents(cents);

    // Assert
    expect(withPlainSpaces(result)).toBe(expected);
  });
});
