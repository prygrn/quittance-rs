import type { CommandErrorCode } from "./api";

/** Titre et explication non technique de chaque échec de commande, repris de la maquette. */
export const COMMAND_ERROR_MESSAGES: Readonly<
  Record<CommandErrorCode, Readonly<{ title: string; text: string }>>
> = {
  validation: {
    title: "Les informations ont été refusées",
    text: "Certaines valeurs n'ont pas été acceptées pour générer la quittance. Vérifiez le formulaire.",
  },
  template: {
    title: "Le modèle de quittance est inutilisable",
    text: "Le modèle choisi n'a pas pu être chargé : il est peut-être incomplet ou endommagé.",
  },
  signature: {
    title: "La signature est introuvable",
    text: "Le fichier de signature indiqué dans votre configuration est absent ou illisible.",
  },
  config: {
    title: "La configuration est incomplète",
    text: "Il manque des informations sur le bailleur ou sur l'envoi des emails dans le fichier de configuration. Complétez-le.",
  },
  pdf: {
    title: "La quittance n'a pas pu être générée",
    text: "La création du document PDF a échoué. Rien n'a été envoyé : vous pouvez réessayer.",
  },
  mail: {
    title: "Le mail n'a pas pu partir",
    text: "Le serveur d'envoi n'a pas répondu ou a refusé le message. Vérifiez votre connexion internet, puis réessayez. Rien n'a été envoyé.",
  },
  unknown: {
    title: "Une erreur inattendue s'est produite",
    text: "L'opération n'a pas abouti et rien n'a été envoyé. Réessayez ; si le problème persiste, redémarrez l'application.",
  },
};
