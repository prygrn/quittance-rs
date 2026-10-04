import type { CommandErrorCode } from "./api";

/** Titre et explication non technique de chaque échec de commande, repris de la maquette. */
export const COMMAND_ERROR_MESSAGES: Readonly<
  Record<CommandErrorCode, Readonly<{ title: string; text: string }>>
> = {
  validation: { title: "", text: "" },
  template: { title: "", text: "" },
  signature: { title: "", text: "" },
  config: { title: "", text: "" },
  pdf: { title: "", text: "" },
  mail: { title: "", text: "" },
  unknown: { title: "", text: "" },
};
