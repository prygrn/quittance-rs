/**
 * Erreur d'une réponse du backend de forme inattendue. Sans `code`, elle est traitée comme
 * `unknown` par l'écran, qui la journalise. Seul le type de la réponse est décrit : son
 * contenu peut porter des données personnelles du locataire.
 */
export function invalidCommandResponseError(details: {
  readonly command: string;
  readonly expected: string;
  readonly response: unknown;
}): Error {
  return new Error(
    `[UI_INVALID_COMMAND_RESPONSE] command ${details.command} returned ${describeValue(details.response)} instead of ${details.expected}`,
  );
}

function describeValue(value: unknown): string {
  if (value === null) {
    return "null";
  }
  return Array.isArray(value) ? "an array" : `a value of type ${typeof value}`;
}
