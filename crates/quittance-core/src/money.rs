/// Montant en centimes d'euro, pour éviter toute erreur d'arrondi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money {
    cents: u64,
}

impl Money {
    pub fn from_cents(cents: u64) -> Self {
        Self { cents }
    }

    pub fn cents(&self) -> u64 {
        self.cents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_cents_when_creating_money_then_it_exposes_the_same_cents() {
        let money = Money::from_cents(123_456);

        assert_eq!(money.cents(), 123_456);
    }

    #[test]
    fn given_zero_cents_when_creating_money_then_it_is_a_valid_amount() {
        let money = Money::from_cents(0);

        assert_eq!(money.cents(), 0);
    }
}
