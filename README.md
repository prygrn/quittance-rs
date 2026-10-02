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
- [agent-kernel](https://github.com/prygrn/agent-kernel) — submodule, en `.agents`

## Architecture

Une feature = un crate (ou un module TS) isolé, testable seul. Les crates de feature ne dépendent que de `quittance-core` ; seule l'app Tauri (`src-tauri/`) les assemble.

| Chemin                       | Rôle                                             |
| ---------------------------- | ------------------------------------------------ |
| `crates/quittance-core`      | Domaine : quittance, parties, période, montants  |
| `crates/quittance-template`  | Sélection du template et rendu HTML              |
| `crates/quittance-signature` | Chargement et validation de l'image de signature |
| `crates/quittance-pdf`       | Conversion HTML → PDF via Chromium headless      |
| `crates/quittance-mailer`    | Construction et envoi de l'email (SMTP)          |
| `crates/french-amount-words` | Montant en toutes lettres, lib autonome          |
| `src-tauri/`                 | App desktop : configuration, orchestration       |
| `src/`                       | Interface (Vanilla TS + Vite)                    |

La documentation du comportement, ce sont les tests : chaque crate et chaque module portent leurs tests unitaires.

## Configuration

Copier `.env.example` en `.env` et le compléter : identité du bailleur, image de signature, chemin de Chromium, serveur SMTP. Le bailleur reçoit chaque quittance en copie cachée, qui sert d'archive.

## Développement

Prérequis : Rust stable, Node ≥ 22.12, Chromium ou Chrome, Docker (Mailpit pour les tests d'intégration), dépendances système de [Tauri sous Linux](https://tauri.app/start/prerequisites/).

```bash
git submodule update --init
make setup        # dépendances npm, hooks git, lien des règles projet
```

| Commande                | Effet                                                           |
| ----------------------- | --------------------------------------------------------------- |
| `make format`           | Formate et corrige automatiquement (rustfmt, prettier, eslint)  |
| `make quality`          | Vérifie format et lint (rustfmt, clippy, prettier, eslint, tsc) |
| `make test-unit`        | Tests unitaires Rust et TS                                      |
| `make test-integration` | Tests d'intégration (Chromium, Mailpit)                         |
| `make test-e2e`         | Tests end-to-end de l'app                                       |
| `make ci`               | Toute la chaîne, comme en CI                                    |

### Workflow test-first

Pour chaque feature : API publique en stubs et suite complète de tests unitaires d'abord (commit `test(<scope>): ...`, tests rouges), puis implémentation jusqu'au vert (commit `feat(<scope>): ...`). Le hook `commit-msg` vérifie le format du message, bloque sur le lint et sur les tests, sauf pour un commit de type `test`.

### CI

GitHub Actions, en deux étages :

1. `quality` puis `unit` : format, lint, build debug, tests unitaires ;
2. `release-integration` : build release, tests d'intégration, tests e2e.

## Statut

Socle en place (workspace, outillage, hooks, CI). Features en cours d'implémentation.
