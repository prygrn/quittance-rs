import { describe, expect, it } from "vitest";

import { FIELD_ERROR_MESSAGES } from "./field-error-messages";

describe("FIELD_ERROR_MESSAGES", () => {
  it("gives every field error a distinct non empty message", () => {
    // Arrange
    const messages = Object.values(FIELD_ERROR_MESSAGES);

    // Act
    const distinctMessages = new Set(messages.filter((message) => message !== ""));

    // Assert
    expect(distinctMessages.size).toBe(messages.length);
  });
});
