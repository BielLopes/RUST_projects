use std::collections::BTreeMap;

use num::{CheckedAdd, CheckedSub, Zero};
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

pub trait Config: crate::system::Config {
    type Balance: Zero + CheckedSub + CheckedAdd + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
    balances: BTreeMap<T::AccountId, T::Balance>,
}

impl<T: Config> Pallet<T> {
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
        }
    }

    pub fn set_balance(&mut self, who: &T::AccountId, balance: T::Balance) {
        self.balances.insert(who.clone(), balance);
    }

    pub fn balance(&self, who: &T::AccountId) -> T::Balance {
        *self.balances.get(who).unwrap_or(&T::Balance::zero())
    }

    pub fn transfer(
        &mut self,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> Result<(), TransferError> {
        let from_balance = self.balance(from);
        let to_balance = self.balance(to);

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or(TransferError::InsufficientBalance)?;
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or(TransferError::OverflowBalance)?;

        self.set_balance(from, new_from_balance);
        self.set_balance(to, new_to_balance);

        Ok(())
    }
}

pub enum Call<'a, T: Config> {
    Transfer {
        to: &'a T::AccountId,
        amount: T::Balance,
    },
}

impl<'a, T: Config> crate::support::Dispatch<'a> for Pallet<T>
where
    T::AccountId: 'a,
{
    type Caller = &'a T::AccountId;
    type Call = Call<'a, T>;

    fn dispatch(
        &mut self,
        caller: Self::Caller,
        call: Self::Call,
    ) -> crate::support::DispatchResult {
        match call {
            Call::Transfer { to, amount } => self.transfer(caller, to, amount)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tets {
    use crate::system;

    use super::*;

    struct TestConfig;

    impl system::Config for TestConfig {
        type AccountId = String;
        type Nonce = u32;
        type BlockNumber = u128;
    }

    impl Config for TestConfig {
        type Balance = u128;
    }

    fn setup() -> (String, String, Pallet<TestConfig>) {
        (String::from("Alice"), String::from("Bob"), Pallet::new())
    }

    #[test]
    fn init_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);

        assert_eq!(pallet.balance(&alice), 100);
        assert_eq!(pallet.balance(&bob), 0);
    }

    #[test]
    fn transfer_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);
        pallet.set_balance(&bob, 100);

        pallet.transfer(&alice, &bob, 50).unwrap();
        assert_eq!(pallet.balance(&alice), 50);
        assert_eq!(pallet.balance(&bob), 150);
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
        assert_eq!(pallet.balance(&alice), 100);
        assert_eq!(pallet.balance(&bob), 100);
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
        assert_eq!(pallet.balance(&alice), u128::MAX);
        assert_eq!(pallet.balance(&bob), 100);
    }
}
