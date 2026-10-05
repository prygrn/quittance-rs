# Journal des modifications

Toutes les modifications notables de ce crate sont consignées dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le crate respecte le
[versionnage sémantique](https://semver.org/lang/fr/). Chaque version publiée correspond au tag
git `french-amount-words-vX.Y.Z`.

## [Unreleased]

### Added

- `number_to_words` : écriture en toutes lettres d'un entier, en graphie traditionnelle.
- `euro_amount_to_words` : écriture en toutes lettres d'un montant en euros exprimé en
  centimes.
- `MAX_VALUE` : plus grande valeur convertible (`999_999_999_999`).
- `AmountWordsError::ValueTooLarge` : erreur renvoyée au-delà de `MAX_VALUE`.
- `AmountWordsError` est `#[non_exhaustive]` : un `match` hors du crate doit prévoir un bras `_`,
  ce qui permet d'ajouter des variantes sans rupture de compatibilité.

[Unreleased]: https://github.com/prygrn/quittance-rs/commits/master/crates/french-amount-words
