#![doc = include_str!("../README.md")]

mod error;
mod euro;
mod number;

pub use error::AmountWordsError;
pub use euro::euro_amount_to_words;
pub use number::number_to_words;

/// Plus grande valeur convertible en lettres : neuf cent quatre-vingt-dix-neuf milliards…
pub const MAX_VALUE: u64 = 999_999_999_999;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
