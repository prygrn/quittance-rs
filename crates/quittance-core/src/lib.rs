//! Types du domaine partagés par toutes les features : quittance, parties, période, montants.
//!
//! Un [`Receipt`] n'existe que validé : il se construit uniquement via [`validate_receipt`].

// Temporaire : API en stubs `todo!()`, retiré à l'implémentation.
#![allow(unused_variables, dead_code)]

mod error;
mod money;
mod party;
mod period;
mod receipt;

pub use error::{PartyError, ReceiptError};
pub use money::Money;
pub use party::Party;
pub use period::RentPeriod;
pub use receipt::{Receipt, ReceiptInput, validate_receipt};
pub use time::Date;
