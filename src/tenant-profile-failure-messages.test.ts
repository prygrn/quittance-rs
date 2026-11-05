import { describe, expect, it } from "vitest";

import { COMMAND_ERROR_MESSAGES } from "./command-error-messages";
import { TENANT_PROFILE_FAILURE_MESSAGES } from "./tenant-profile-failure-messages";

describe("TENANT_PROFILE_FAILURE_MESSAGES", () => {
  it("gives every tenant profile failure a title and an explanation", () => {
    // Arrange
    const messages = Object.values(TENANT_PROFILE_FAILURE_MESSAGES);

    // Act
    const incomplete = messages.filter(({ title, text }) => title === "" || text === "");

    // Assert
    expect(incomplete).toEqual([]);
  });

  it("gives every tenant profile failure a title distinct from the command errors", () => {
    // Arrange
    const titles = [
      ...Object.values(TENANT_PROFILE_FAILURE_MESSAGES),
      ...Object.values(COMMAND_ERROR_MESSAGES),
    ].map(({ title }) => title);

    // Act
    const distinctTitles = new Set(titles);

    // Assert
    expect(distinctTitles.size).toBe(titles.length);
  });
});
