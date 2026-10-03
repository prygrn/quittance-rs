/** Syntaxe d'adresse de `party.rs` : sous-ensemble de la RFC 5322 sans forme entre guillemets. */
const EMAIL_SYNTAX = {
  ADDRESS_SEPARATOR: "@",
  DOT: ".",
  // Atome `atext` de la RFC 5322 (section 3.2.3), restreint à l'ASCII.
  LOCAL_PART_ATOM: /^[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+$/,
  // Label non vide de lettres, chiffres et tirets, sans tiret en début ni en fin.
  DOMAIN_LABEL: /^[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?$/,
  MINIMUM_DOMAIN_LABEL_COUNT: 2,
} as const;

/** Reflet de la vérification de `party.rs` : sous-ensemble ASCII de la RFC 5322. */
export function isWellFormedEmail(email: string): boolean {
  const separatorIndex = email.indexOf(EMAIL_SYNTAX.ADDRESS_SEPARATOR);
  if (separatorIndex < 0) {
    return false;
  }
  const localPart = email.slice(0, separatorIndex);
  const domain = email.slice(separatorIndex + EMAIL_SYNTAX.ADDRESS_SEPARATOR.length);
  return isWellFormedLocalPart(localPart) && isWellFormedDomain(domain);
}

function isWellFormedLocalPart(localPart: string): boolean {
  return localPart.split(EMAIL_SYNTAX.DOT).every((atom) => EMAIL_SYNTAX.LOCAL_PART_ATOM.test(atom));
}

function isWellFormedDomain(domain: string): boolean {
  const labels = domain.split(EMAIL_SYNTAX.DOT);
  return (
    labels.length >= EMAIL_SYNTAX.MINIMUM_DOMAIN_LABEL_COUNT &&
    labels.every((label) => EMAIL_SYNTAX.DOMAIN_LABEL.test(label))
  );
}
