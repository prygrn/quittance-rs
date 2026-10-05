# quittance-rs

Générateur de quittances de loyer, pour usage personnel (bailleur particulier).

## Fonctionnement

- Écran unique : sélection d'un template de quittance + saisie des champs (locataire, logement, période, montant, etc.)
- Génération du document à partir du template (HTML/CSS → PDF)
- Preview du document avant envoi
- Signature manuscrite protégée contre la copie (voir ci-dessous)
- Envoi automatique au locataire par email (SMTP)

L'outil ne produit que des quittances, c'est-à-dire des attestations de paiement intégral du loyer et des charges d'une période. Un paiement partiel appelle un simple reçu, hors du périmètre de quittance-rs.

## Signature protégée contre la copie

Une quittance porte la signature du bailleur et part chez un tiers. Dans un PDF classique, l'image de signature y est incorporée telle quelle : n'importe qui peut l'extraire en un clic, en pleine résolution et avec sa transparence, puis la coller sur un autre document.

La signature est un fichier image dont le chemin est fixé dans `.env` ; l'écran ne propose pas d'import. quittance-rs ne met jamais l'image d'origine dans le PDF. Avant chaque envoi, la signature est :

- **fusionnée avec une mention propre à la quittance** (période, locataire), écrite en travers de la signature et aplatie dans la même image : une signature extraite reste liée à cette quittance, ce qui rend une réutilisation facile à repérer ;
- **dégradée pour la copie** : résolution limitée au strict nécessaire pour l'impression, fond blanc à la place de la transparence. Elle s'intègre moins bien à un autre document.

Une capture d'écran reste possible ; elle ne donne qu'une image marquée et de qualité réduite.

## Évolutions prévues

- **Signature électronique certifiée (PAdES)** : preuve d'intégrité et d'origine vérifiable. Un certificat scelle le PDF et toute modification devient détectable. Elle remplacera à terme la signature image.

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

Copier `.env.example` en `.env` et le compléter : identité du bailleur, chemin de l'image de signature, chemin de Chromium, serveur SMTP. Le bailleur reçoit chaque quittance en copie cachée, qui sert d'archive.

### Emplacement du fichier `.env`

L'application lit le fichier `.env` placé **dans le même dossier que son exécutable** (en développement : `target/debug/` ou `target/release/`). Elle le relit à chaque aperçu et à chaque envoi.

- Une variable déjà définie dans l'environnement au lancement l'emporte sur celle du fichier.
- Un fichier absent n'est pas une erreur : seules les variables manquantes sont signalées.
- L'aperçu n'exige que le bailleur (`LANDLORD_*`) et `SIGNATURE_PATH`. `CHROME_PATH` et `SMTP_*` ne sont exigés qu'à l'envoi.
- Une valeur entre guillemets simples est prise littéralement ; entre guillemets doubles ou sans guillemets, `$NOM` est remplacé par une variable et `\` échappe le caractère suivant. Écrire donc `SMTP_PASSWORD` entre guillemets simples (une apostrophe s'y écrit `'\''`).

## Développement

Prérequis : Rust stable, Node ≥ 22.12, Chromium ou Chrome, Docker (Mailpit pour les tests d'intégration), dépendances système de [Tauri sous Linux](https://tauri.app/start/prerequisites/).

```bash
git submodule update --init
make system-deps  # bibliothèques système de Tauri (Debian, Ubuntu ; sudo)
make setup        # dépendances npm, hooks git (.githooks)
```

| Commande                | Effet                                                           |
| ----------------------- | --------------------------------------------------------------- |
| `make format`           | Formate et corrige automatiquement (rustfmt, prettier, eslint)  |
| `make quality`          | Vérifie format et lint (rustfmt, clippy, prettier, eslint, tsc) |
| `make test-unit`        | Tests unitaires Rust et TS                                      |
| `make test`             | Alias de `make test-unit`, appelé par le hook du kernel         |
| `make test-integration` | Tests d'intégration (Chromium, Mailpit)                         |
| `make test-e2e`         | Tests end-to-end de l'app                                       |
| `make msrv-check`       | Compile avec le `rust-version` du `Cargo.toml` (rustup requis)  |
| `make ci`               | Toute la chaîne, comme en CI                                    |

### Lancer l'application

```bash
npx tauri dev                  # front servi par Vite (port 5173), .env dans target/debug/
npx tauri build --no-bundle    # exécutable target/release/quittance-app, .env à côté
```

### Content Security Policy

La CSP de `src-tauri/tauri.conf.json` n'autorise que les scripts et styles de l'app et les appels IPC de Tauri. L'aperçu est une `iframe srcdoc` qui hérite de cette CSP : `style-src` accepte donc les styles inline et `img-src` les images `data:` (signature) du HTML de la quittance. `index.html` ne doit contenir aucune balise `<style>` : Tauri ajouterait un nonce à `style-src`, ce qui désactive `'unsafe-inline'` et casse l'aperçu. La CSP ne s'applique qu'à l'app construite ; avec `tauri dev`, la page vient directement du serveur Vite.

### Workflow test-first

Pour chaque feature : API publique en stubs et suite complète de tests unitaires d'abord (commit `test(<scope>): ...`, tests rouges), puis implémentation jusqu'au vert (commit `feat(<scope>): ...`). Les hooks `commit-msg` et `pre-push` de `.githooks/` sont des wrappers suivis par git : ils lancent les hooks du kernel de la worktree courante (`.agents/hooks/`) et bloquent si le submodule est absent. Le hook `commit-msg` vérifie le format du message, lance `make quality` et `make test`, et bloque en cas d'échec ; un commit de type `test` tolère des tests rouges. Le hook `pre-push` refuse tout push sur `main` ou `master`.

### Règles des agents

`AGENTS.md` est généré par `.agents/scripts/compile-agents` à partir des règles du kernel et de `rules/project/` ; `CLAUDE.md` pointe vers lui. Après toute modification de `rules/project/` ou montée de version du kernel, relancer le script et commiter `AGENTS.md` : `make quality` échoue s'il n'est pas à jour.

### CI

GitHub Actions, en deux étages :

1. `quality` puis `unit` : format, lint, build debug, tests unitaires ;
2. `release-integration` : build release, tests d'intégration, tests e2e.

## Statut

Socle en place (workspace, outillage, hooks, CI). Features en cours d'implémentation.
