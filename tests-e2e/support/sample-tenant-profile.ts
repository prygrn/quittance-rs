import { SAMPLE_RECEIPT } from "./sample-receipt";

const entries: typeof SAMPLE_RECEIPT.entries = SAMPLE_RECEIPT.entries;
const ID: string = "jeanne-martin";
const TEMPLATE_ID: string = "standard";
const REPLACEMENT_RENT_AMOUNT: string = "700";

/** Fichier de la fiche tirée de la quittance des scénarios, au format du README. */
function tenantProfileFileContents(rent: string): string {
  return [
    `template_id = "${TEMPLATE_ID}"`,
    `tenant_name = "${entries.tenantName}"`,
    `tenant_address = "${entries.tenantAddress}"`,
    `tenant_email = "${entries.tenantEmail}"`,
    `property_address = "${entries.propertyAddress}"`,
    `rent = "${rent}"`,
    `charges = "${entries.chargesAmount}"`,
  ]
    .map((line) => `${line}\n`)
    .join("");
}

const FILE_CONTENTS: string = tenantProfileFileContents("650,00");
const REPLACED_FILE_CONTENTS: string = tenantProfileFileContents("700,00");

/**
 * Fiche locataire enregistrée depuis la quittance des scénarios : son fichier, les champs
 * qu'elle préremplit, puis le loyer qui la remplace et le fichier qui en résulte.
 */
export const SAMPLE_TENANT_PROFILE = {
  id: ID,
  label: entries.tenantName,
  file: {
    fileName: `${ID}.toml`,
    knownContents: [FILE_CONTENTS, REPLACED_FILE_CONTENTS],
  },
  fileContents: FILE_CONTENTS,
  // Champs préremplis par la fiche : les montants y sont réécrits avec deux décimales.
  prefilledEntries: {
    tenantName: entries.tenantName,
    tenantAddress: entries.tenantAddress,
    tenantEmail: entries.tenantEmail,
    propertyAddress: entries.propertyAddress,
    rentAmount: "650,00",
    chargesAmount: entries.chargesAmount,
  },
  replacementRentAmount: REPLACEMENT_RENT_AMOUNT,
  replacedFileContents: REPLACED_FILE_CONTENTS,
} as const;
