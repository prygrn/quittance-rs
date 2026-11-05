/**
 * Titre et explication des échecs propres aux fiches locataire, que le libellé générique
 * du code d'erreur décrirait mal.
 */
export const TENANT_PROFILE_FAILURE_MESSAGES: Readonly<
  Record<"saveRejected" | "listNotRefreshed", Readonly<{ title: string; text: string }>>
> = {
  saveRejected: { title: "", text: "" },
  listNotRefreshed: { title: "", text: "" },
};
