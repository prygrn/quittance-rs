import { afterEach, describe, expect, it, vi } from "vitest";

import { toLocalIsoDate } from "./to-local-iso-date";

describe("toLocalIsoDate", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it("pads the month and the day with a leading zero", () => {
    // Arrange
    const date = new Date(2026, 0, 5, 12, 0);

    // Act
    const isoDate = toLocalIsoDate(date);

    // Assert
    expect(isoDate).toBe("2026-01-05");
  });

  it.each([
    {
      timeZone: "Europe/Paris",
      description: "just after midnight ahead of UTC",
      instant: "2026-10-05T22:05:00Z",
      expected: "2026-10-06",
    },
    {
      timeZone: "Europe/Paris",
      description: "just after midnight on new year's day",
      instant: "2026-12-31T23:30:00Z",
      expected: "2027-01-01",
    },
    {
      timeZone: "America/New_York",
      description: "just before midnight behind UTC",
      instant: "2026-10-06T03:30:00Z",
      expected: "2026-10-05",
    },
  ])("reads the local day in $timeZone $description", ({ timeZone, instant, expected }) => {
    // Arrange
    vi.stubEnv("TZ", timeZone);
    const date = new Date(instant);

    // Act
    const isoDate = toLocalIsoDate(date);

    // Assert
    expect(isoDate).toBe(expected);
  });

  it("rejects an invalid date", () => {
    // Arrange
    const date = new Date(Number.NaN);

    // Act
    const convert = (): string => toLocalIsoDate(date);

    // Assert
    expect(convert).toThrow("[UI_INVALID_CLOCK_DATE]");
  });
});
