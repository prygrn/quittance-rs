# quittance-rs

Générateur de quittances de loyer, pour usage personnel (bailleur particulier).

## Fonctionnement

- Écran unique : sélection d'un template de quittance + saisie des champs (locataire, logement, période, montant, etc.)
- Génération du document à partir du template (HTML/CSS → PDF)
- Preview du document avant envoi
- Signature électronique (image de signature superposée au PDF)
- Envoi automatique au locataire par email (SMTP)

## Stack

- [Tauri](https://tauri.app/) — IHM desktop locale (pas d'hébergement, lancement en localhost uniquement)
- Rust (backend Tauri)
- [agent-kernel](https://github.com/prygrn/agent-kernel) — submodule, en `vendor/agent-kernel`

## Statut

Projet en démarrage. Architecture et stack validées, implémentation à venir.
