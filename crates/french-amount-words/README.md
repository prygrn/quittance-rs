# french-amount-words

Écriture en toutes lettres des nombres entiers et des montants en euros, en français, selon la
graphie traditionnelle.

Le crate est autonome : il ne dépend d'aucune autre bibliothèque que la bibliothèque standard.
Il sert typiquement à reporter une somme en lettres sur une quittance, une facture ou un chèque.

## Installation

```sh
cargo add french-amount-words
```

## Usage

```rust
use french_amount_words::{AmountWordsError, euro_amount_to_words, number_to_words};

fn main() -> Result<(), AmountWordsError> {
    assert_eq!(number_to_words(1234)?, "mille deux cent trente-quatre");
    assert_eq!(number_to_words(80)?, "quatre-vingts");
    assert_eq!(number_to_words(2_000_000)?, "deux millions");

    // Les montants sont exprimés en centimes pour éviter les flottants.
    assert_eq!(
        euro_amount_to_words(123_456)?,
        "mille deux cent trente-quatre euros et cinquante-six centimes"
    );
    assert_eq!(euro_amount_to_words(100)?, "un euro");
    assert_eq!(euro_amount_to_words(50)?, "cinquante centimes");
    assert_eq!(euro_amount_to_words(100_000_000)?, "un million d'euros");
    Ok(())
}
```

## Borne maximale

La constante `MAX_VALUE` vaut `999_999_999_999` (neuf cent quatre-vingt-dix-neuf milliards…).

- `number_to_words` renvoie `AmountWordsError::ValueTooLarge` au-delà de `MAX_VALUE`.
- `euro_amount_to_words` renvoie la même erreur quand la partie en euros dépasse `MAX_VALUE` ;
  le champ `value` de l'erreur contient alors la partie en euros, et non l'entrée en centimes.

## Règles d'orthographe appliquées

La graphie suivie est la graphie traditionnelle décrite par l'Académie française, et non celle
des rectifications orthographiques de 1990 (qui lient tous les éléments par des traits d'union).

- Trait d'union uniquement entre les éléments inférieurs à cent : `vingt-deux`, `dix-sept`,
  `quatre-vingt-dix-neuf`, mais `deux cent un`, `mille cent`.
- « et » sans trait d'union pour 21, 31, 41, 51, 61 et 71 : `vingt et un`, `soixante et onze`.
  Pas de « et » pour 81 et 91 : `quatre-vingt-un`, `quatre-vingt-onze`.
- `vingt` et `cent` prennent un « s » quand ils sont multipliés et terminent le nombre :
  `quatre-vingts`, `deux cents`, mais `quatre-vingt-un`, `deux cent un`.
- Devant `mille`, adjectif numéral, ils restent invariables : `quatre-vingt mille`,
  `deux cent mille`.
- `mille` est invariable et n'est jamais précédé de « un » : `mille`, `deux mille`.
  De même, `cent` n'est jamais précédé de « un ».
- `million` et `milliard` sont des noms : ils prennent la marque du pluriel et n'empêchent pas
  l'accord de `vingt` et `cent` : `un million`, `deux millions`, `quatre-vingts millions`,
  `deux cents milliards`.
- Montants en euros :
  - `euro` et `centime` restent au singulier pour zéro et un : `zéro euro`, `un euro`,
    `un centime` ;
  - « zéro centime » n'est jamais écrit (`deux euros`) et « zéro euro » non plus quand il y a
    des centimes (`cinquante centimes`) ;
  - après `million` ou `milliard` terminant la partie en euros, on écrit « d'euros » :
    `un million d'euros`, mais `un million deux cents euros`.

## Licence

MIT, voir le fichier `LICENSE`.
