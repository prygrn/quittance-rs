/**
 * Titre et explication des échecs propres aux fiches locataire, que le libellé générique
 * du code d'erreur décrirait mal.
 */
export const TENANT_PROFILE_FAILURE_MESSAGES: Readonly<
  Record<"saveRejected" | "listNotRefreshed", Readonly<{ title: string; text: string }>>
> = {
  saveRejected: {
    title: "La fiche a été refusée",
    text: "Certaines valeurs n'ont pas été acceptées pour enregistrer la fiche. Vérifiez le formulaire, puis réessayez.",
  },
  listNotRefreshed: {
    title: "Fiche enregistrée, liste non rechargée",
    text: "La fiche a bien été enregistrée, mais la liste des fiches n'a pas pu être rechargée. Redémarrez l'application pour la retrouver dans la liste.",
  },
};
