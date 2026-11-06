/** Valeur du champ propre `field` de `value`, ou `undefined` s'il n'en a pas (champ hérité exclu). */
export function readOwnField(candidate: {
  readonly value: unknown;
  readonly field: string;
}): unknown {
  const { value, field } = candidate;
  if (typeof value !== "object" || value === null || !Object.hasOwn(value, field)) {
    return undefined;
  }
  return (value as Readonly<Record<string, unknown>>)[field];
}
