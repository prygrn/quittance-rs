/** Option d'une liste déroulante, créée dans le document de la liste qui l'accueille. */
export function createOption(options: {
  readonly ownerDocument: Document;
  readonly value: string;
  readonly label: string;
}): HTMLOptionElement {
  const option = options.ownerDocument.createElement("option");
  option.value = options.value;
  option.textContent = options.label;
  return option;
}
