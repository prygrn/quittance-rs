# Journal des modifications

Toutes les modifications notables de l'application sont consignées dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et l'application respecte le
[versionnage sémantique](https://semver.org/lang/fr/). Chaque version correspond au tag git `vX.Y.Z`.

## [Unreleased]

## [1.0.1] - 2026-10-05

### Changed

- Le montant en toutes lettres utilise le crate [`french-amount-words`](https://crates.io/crates/french-amount-words)
  0.1.1 publié sur crates.io, au lieu de sa copie locale dans `crates/french-amount-words`, supprimée.

## [1.0.0] - 2026-10-05

### Added

- Saisie d'une quittance de loyer : locataire, logement, période, loyer, charges, date de paiement.
- Aperçu de la quittance avant envoi, avec la signature protégée contre la copie.
- Envoi par email au locataire, avec le PDF en pièce jointe et le bailleur en copie cachée.
- Configuration du bailleur, de la signature et du serveur SMTP dans un fichier `.env` placé à côté
  de l'exécutable.

[Unreleased]: https://github.com/prygrn/quittance-rs/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/prygrn/quittance-rs/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/prygrn/quittance-rs/releases/tag/v1.0.0
