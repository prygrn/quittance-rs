type JsonRecord = Readonly<Record<string, unknown>>;

interface FieldLookup {
  readonly record: JsonRecord;
  readonly name: string;
  /** Provenance du JSON, reprise dans le message d'erreur. */
  readonly context: string;
}

function requireRecord(options: { readonly value: unknown; readonly context: string }): JsonRecord {
  const { value, context } = options;
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(
      `[E2E_UNEXPECTED_JSON] ${context} should be a JSON object, got ${JSON.stringify(value)}`,
    );
  }
  return value as JsonRecord;
}

function requireField<T>(
  lookup: FieldLookup & {
    readonly expected: string;
    readonly isExpected: (value: unknown) => value is T;
  },
): T {
  const value = lookup.record[lookup.name];
  if (!lookup.isExpected(value)) {
    throw new Error(
      `[E2E_UNEXPECTED_JSON] ${lookup.context}.${lookup.name} should be ${lookup.expected}, got ${JSON.stringify(value)}`,
    );
  }
  return value;
}

/**
 * Lecture validée des réponses JSON de Mailpit : toute forme inattendue lève une erreur
 * qui nomme le champ en cause, au lieu de laisser une assertion échouer plus loin.
 */
export const JSON_READER = {
  requireRecord,
  requireString: (lookup: FieldLookup): string =>
    requireField({
      ...lookup,
      expected: "a string",
      isExpected: (value: unknown): value is string => typeof value === "string",
    }),
  requireNumber: (lookup: FieldLookup): number =>
    requireField({
      ...lookup,
      expected: "a number",
      isExpected: (value: unknown): value is number => typeof value === "number",
    }),
  requireArray: (lookup: FieldLookup): readonly unknown[] =>
    requireField({
      ...lookup,
      expected: "an array",
      isExpected: (value: unknown): value is readonly unknown[] => Array.isArray(value),
    }),
} as const;
