import { describe, expect, it } from "vitest";

import type { CommandErrorCode } from "./api";
import { toCommandErrorCode } from "./to-command-error-code";

describe("toCommandErrorCode", () => {
  it.each<CommandErrorCode>([
    "validation",
    "template",
    "signature",
    "pdf",
    "mail",
    "config",
    "unknown",
  ])("reads the %s code from a command rejection", (code: CommandErrorCode) => {
    // Arrange
    const reason = { code, message: "détail technique du backend" };

    // Act
    const result = toCommandErrorCode(reason);

    // Assert
    expect(result).toBe(code);
  });

  it.each<{ description: string; reason: unknown }>([
    { description: "an unrecognized code", reason: { code: "network" } },
    { description: "a non string code", reason: { code: 42 } },
    { description: "an inherited code", reason: Object.create({ code: "mail" }) },
    { description: "an error without code", reason: new Error("boom") },
    { description: "a plain string", reason: "mail" },
    { description: "null", reason: null },
    { description: "undefined", reason: undefined },
  ])("maps $description to unknown", ({ reason }) => {
    // Arrange & Act
    const result = toCommandErrorCode(reason);

    // Assert
    expect(result).toBe("unknown");
  });
});
