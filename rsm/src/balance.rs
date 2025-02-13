use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Overflow when added to balance")]
    OverflowBalance,
}

// Simple way to transform error to string implementing the Display trait:
// use std::fmt::Display;

// #[derive(Debug)]
// pub enum TransferError {
//     InsufficientBalance,
//     OverflowBalance,
// }
//
// impl Display for TransferError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             TransferError::InsufficientBalance => write!(f, "Insufficient balance"),
//             TransferError::OverflowBalance => write!(f, "Overflow when added to balance"),
//         }
//     }
// }

type AccountId = String;
type Balance = u128;

#[derive(Debug)]
pub struct Pallet {
    balances: BTreeMap<AccountId, Balance>,
}

impl Pallet {
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
        }
    }

    pub fn set_balance(&mut self, who: &AccountId, balance: Balance) {
        self.balances.insert(who.clone(), balance);
    }

    pub fn balance(&self, who: &AccountId) -> &Balance {
        self.balances.get(who).unwrap_or(&0)
    }

    pub fn transfer(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        amount: Balance,
    ) -> Result<(), TransferError> {
        let from_balance = self.balance(from);
        let to_balance = self.balance(to);

        let new_from_balance = from_balance
            .checked_sub(amount)
            .ok_or(TransferError::InsufficientBalance)?;
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(TransferError::OverflowBalance)?;

        self.set_balance(from, new_from_balance);
        self.set_balance(to, new_to_balance);

        Ok(())
    }
}

#[cfg(test)]
mod tets {
    use super::*;

    fn setup() -> (String, String, Pallet) {
        (String::from("Alice"), String::from("Bob"), Pallet::new())
    }

    #[test]
    fn init_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);

        assert_eq!(pallet.balance(&alice), &100);
        assert_eq!(pallet.balance(&bob), &0);
    }

    #[test]
    fn transfer_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);
        pallet.set_balance(&bob, 100);

        pallet.transfer(&alice, &bob, 50).unwrap();
        assert_eq!(pallet.balance(&alice), &50);
        assert_eq!(pallet.balance(&bob), &150);
    }

    #[test]
    fn transfer_insufficient_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);
        pallet.set_balance(&bob, 100);

        let result = pallet.transfer(&alice, &bob, 150);

        assert_eq!(
            result.unwrap_err().to_string(),
            TransferError::InsufficientBalance.to_string()
        );
        assert_eq!(pallet.balance(&alice), &100);
        assert_eq!(pallet.balance(&bob), &100);
    }

    #[test]
    fn transfer_overflow_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, u128::MAX);
        pallet.set_balance(&bob, 100);

        let result = pallet.transfer(&bob, &alice, 1);

        assert_eq!(
            result.unwrap_err().to_string(),
            TransferError::OverflowBalance.to_string()
        );
        assert_eq!(pallet.balance(&alice), &u128::MAX);
        assert_eq!(pallet.balance(&bob), &100);
    }
}
