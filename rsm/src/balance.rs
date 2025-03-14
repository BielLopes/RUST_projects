use std::collections::BTreeMap;

use num::{CheckedAdd, CheckedSub, Zero};

pub trait Config: crate::system::Config {
    type Balance: Zero + CheckedSub + CheckedAdd + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
    balances: BTreeMap<T::AccountId, T::Balance>,
}

#[macros::call]
impl<T: Config> Pallet<T> {
    pub fn transfer(
        &mut self,
        caller: T::AccountId,
        to: T::AccountId,
        amount: T::Balance,
    ) -> crate::support::DispatchResult {
        let from_balance = self.balance(&caller);
        let to_balance = self.balance(&to);

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or("Insufficient balance.")?;
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or("Overflow when adding to balance.")?;

        self.set_balance(&caller, new_from_balance);
        self.set_balance(&to, new_to_balance);

        Ok(())
    }
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
        *self.balances.get(&who).unwrap_or(&T::Balance::zero())
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

        pallet.transfer(alice.clone(), bob.clone(), 50).unwrap();
        assert_eq!(pallet.balance(&alice), 50);
        assert_eq!(pallet.balance(&bob), 150);
    }

    #[test]
    fn transfer_insufficient_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, 100);
        pallet.set_balance(&bob, 100);

        let result = pallet.transfer(alice.clone(), bob.clone(), 150);

        assert_eq!(result.unwrap_err(), "Insufficient balance.");
        assert_eq!(pallet.balance(&alice), 100);
        assert_eq!(pallet.balance(&bob), 100);
    }

    #[test]
    fn transfer_overflow_balance() {
        let (alice, bob, mut pallet) = setup();

        pallet.set_balance(&alice, u128::MAX);
        pallet.set_balance(&bob, 100);

        let result = pallet.transfer(bob.clone(), alice.clone(), 1);

        assert_eq!(result.unwrap_err(), "Overflow when adding to balance.");
        assert_eq!(pallet.balance(&alice), u128::MAX);
        assert_eq!(pallet.balance(&bob), 100);
    }
}
