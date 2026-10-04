# Prompt Claude Design — quittance-rs

Conçois la maquette de l'écran unique d'une application desktop de génération de quittances de loyer.

## Contexte

- Application desktop **locale** (Tauri), pour un **bailleur particulier**, en usage personnel. Pas de compte, pas de connexion, pas d'hébergement.
- Un seul écran. Le bailleur saisit les informations d'une quittance, en vérifie l'aperçu, puis l'envoie par email au locataire. Le PDF est joint au mail, et le bailleur reçoit une copie cachée qui lui sert d'archive.
- L'identité du bailleur (nom, adresse, email, ville) et sa signature sont configurées hors de l'écran, dans un fichier de configuration. **Elles ne se saisissent pas dans l'interface** : ne prévois ni champ pour le bailleur ni import de signature.
- Interface entièrement en **français**.

## Éléments obligatoires

1. **Sélecteur de modèle de quittance** (liste déroulante). Un seul modèle pour l'instant, « Standard », mais la liste doit pouvoir en accueillir d'autres.
2. **Formulaire** : les champs exacts ci-dessous, **tous obligatoires**, chacun avec son libellé visible.

   | Libellé                | Type             | Remarque                                   |
   | ---------------------- | ---------------- | ------------------------------------------ |
   | Nom du locataire       | texte            |                                            |
   | Adresse du locataire   | texte multiligne |                                            |
   | Email du locataire     | email            | il figure sur la quittance (preuve légale) |
   | Adresse du logement    | texte multiligne |                                            |
   | Début de période       | date             |                                            |
   | Fin de période         | date             |                                            |
   | Loyer (€)              | montant          | saisie libre, ex. « 650 » ou « 1 234,56 »  |
   | Forfait de charges (€) | montant          | idem                                       |
   | Date de paiement       | date             |                                            |

   Ajoute un **total (loyer + forfait de charges)**, affiché en lecture seule sous les montants.

3. **Bouton « Aperçu »** : génère l'aperçu de la quittance.
4. **Zone d'aperçu** : affiche la quittance, une page A4 en HTML avec la signature déjà intégrée, dans un cadre de type iframe. Prévois-la assez grande pour que la page soit lisible : réduite à l'échelle, avec défilement vertical.
5. **Bouton « Envoyer »** : envoie la quittance au locataire. **Il n'est actif qu'après un aperçu réussi.** Toute modification du formulaire ou du modèle invalide l'aperçu, et le bouton redevient inactif.

## États à maquetter, un écran ou une variante par état

1. **Vide** : formulaire vierge, zone d'aperçu avec un message d'invitation, « Envoyer » désactivé.
2. **Erreurs de validation par champ** : un message sous le champ concerné, avec le champ mis en évidence. Les messages à prévoir :
   - champ obligatoire manquant ;
   - email invalide ;
   - date invalide ;
   - fin de période antérieure au début, signalé sur « Fin de période » ;
   - montant invalide, montant négatif, plus de deux décimales, montant trop élevé.

   Toutes les erreurs s'affichent en même temps.

3. **Aperçu en cours** : indicateur de chargement dans la zone d'aperçu.
4. **Aperçu chargé** : la quittance s'affiche, « Envoyer » est actif.
5. **Envoi en cours** : le **formulaire entier est verrouillé** (champs et sélecteur désactivés) et le bouton indique l'envoi en cours.
6. **Succès** : confirmation claire que le mail est parti, avec l'email du destinataire.
7. **Échec** : un message d'erreur compréhensible et non technique selon la cause :
   - données refusées ;
   - problème de modèle ;
   - problème de signature ;
   - échec de génération du PDF ;
   - échec d'envoi du mail (serveur SMTP) ;
   - configuration incomplète ;
   - erreur inconnue.

   Après un échec d'**envoi**, un bouton « Réessayer l'envoi » permet de relancer **sans refaire l'aperçu**. Après un échec d'**aperçu**, il faut relancer l'aperçu.

## Contraintes techniques

- Livrable : **HTML + CSS statiques**, réutilisables tels quels avec du **TypeScript sans framework** (pas de React, Vue, etc.). Pas de JavaScript nécessaire dans la maquette.
- **Aucune ressource externe** : ni police web, ni CDN, ni image distante. Utilise des polices système.
- Structure sémantique : `<form>`, `<label for>`, `<fieldset>` si utile, `<button>`. Ajoute des identifiants stables sur les champs, les boutons et la zone d'aperçu, pour brancher la logique.
- Fenêtre desktop d'environ **1200 × 800 px**. Formulaire et aperçu côte à côte de préférence.
- **Thème clair** uniquement.

## Accessibilité

- Chaque champ a un libellé associé. Les erreurs sont reliées à leur champ (`aria-describedby`, `aria-invalid`).
- Contrastes conformes WCAG AA.
- Navigation complète au clavier, avec un ordre logique et un focus visible.
- Les états de chargement et de succès ou d'échec sont annoncés aux lecteurs d'écran (`aria-live`).

## Ton visuel

Sobre, administratif mais pas austère. On doit avoir confiance dans un document qui part chez un tiers. Pas de décor inutile.
