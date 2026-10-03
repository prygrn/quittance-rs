import { describe, expect, it } from "vitest";

import { isWellFormedEmail } from "./email";

// Mêmes jeux de données que les tests de `party.rs`, pour garder client et core alignés.
describe("isWellFormedEmail", () => {
  it.each([
    "a@b.co",
    "jeanne.martin@example.fr",
    "jeanne.martin+loyer@mail.example.fr",
    "jeanne@mon-domaine.fr",
    "o'neil@example.fr",
    "!#$%&'*+/=?^_`{|}~-@example.fr",
  ])("accepts %j", (email: string) => {
    // Arrange & Act
    const isWellFormed = isWellFormedEmail(email);

    // Assert
    expect(isWellFormed).toBe(true);
  });

  it.each([
    "",
    "jeanne.martin",
    "@example.fr",
    "jeanne@",
    "jeanne@example",
    "jeanne@@example.fr",
    "jeanne@exam@ple.fr",
    "jeanne martin@example.fr",
    " jeanne@example.fr",
    "jeanne@.fr",
    "jeanne@example.",
    "jean,ne@example.fr",
    "a<b>@example.fr",
    "jean(ne)@example.fr",
    'jean"ne@example.fr',
    "jean;ne@example.fr",
    ".jeanne@example.fr",
    "jeanne.@example.fr",
    "jean..ne@example.fr",
    "jeanne@example..fr",
    "jeanne@-example.fr",
    "jeanne@example-.fr",
    "jeanne@exa_mple.fr",
    "jeanne@example.f!r",
    "jeanné@exemple.fr",
    "jeanne@exémple.fr",
  ])("rejects %j", (email: string) => {
    // Arrange & Act
    const isWellFormed = isWellFormedEmail(email);

    // Assert
    expect(isWellFormed).toBe(false);
  });
});
