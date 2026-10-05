import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "target/",
      "dist/",
      "node_modules/",
      ".agents/",
      "src-tauri/target/",
      "design/mockup/",
    ],
  },
  eslint.configs.recommended,
  tseslint.configs.strict,
  {
    // Conventions de nommage du kernel (rules/always/conventions.md) appliquées au TypeScript.
    rules: {
      "@typescript-eslint/naming-convention": [
        "error",
        { selector: "default", format: ["camelCase"] },
        {
          selector: "variable",
          modifiers: ["const", "global"],
          format: ["camelCase", "UPPER_CASE"],
        },
        { selector: "typeLike", format: ["PascalCase"] },
        { selector: "enumMember", format: ["PascalCase"] },
        { selector: "objectLiteralProperty", format: null },
        // Clé imposée par un protocole externe, comme la capacité WebDriver `tauri:options`.
        { selector: "typeProperty", modifiers: ["requiresQuotes"], format: null },
        { selector: "import", format: null },
      ],
    },
  },
);
