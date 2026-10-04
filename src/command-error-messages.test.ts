import { describe, expect, it } from "vitest";

import { COMMAND_ERROR_MESSAGES } from "./command-error-messages";

describe("COMMAND_ERROR_MESSAGES", () => {
  it("gives every command error a title and an explanation", () => {
    // Arrange
    const messages = Object.values(COMMAND_ERROR_MESSAGES);

    // Act
    const incomplete = messages.filter(({ title, text }) => title === "" || text === "");

    // Assert
    expect(incomplete).toEqual([]);
  });

  it("gives every command error its own title", () => {
    // Arrange
    const titles = Object.values(COMMAND_ERROR_MESSAGES).map(({ title }) => title);

    // Act
    const distinctTitles = new Set(titles);

    // Assert
    expect(distinctTitles.size).toBe(titles.length);
  });
});
