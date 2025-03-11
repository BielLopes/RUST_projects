use num::{CheckedAdd, CheckedSub, One, Zero};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Overflow when incrementing block number")]
    OverflowIncrementBlockNumber,
    #[error("Overflow when incrementing nonce")]
    OverflowIncrementNonce,
}

pub trait Config {
    type AccountId: Ord + Clone;
    type BlockNumber: Zero + One + CheckedAdd + Copy;
    type Nonce: Zero + One + CheckedSub + CheckedAdd + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
    bloc_number: T::BlockNumber,
    nonce: BTreeMap<T::AccountId, T::Nonce>, // Numver of transactions someone make on the blockchain
}

impl<T: Config> Pallet<T> {
    pub fn new() -> Self {
        Self {
            bloc_number: T::BlockNumber::zero(),
            nonce: BTreeMap::new(),
        }
    }

    pub fn block_number(&self) -> T::BlockNumber {
        self.bloc_number
    }

    pub fn increment_block_number(&mut self) -> Result<(), BlockError> {
        self.bloc_number = self
            .bloc_number
            .checked_add(&T::BlockNumber::one())
            .ok_or(BlockError::OverflowIncrementBlockNumber)?;

        Ok(())
    }

    pub fn increment_nonce(&mut self, who: &T::AccountId) -> Result<(), BlockError> {
        let binding = T::Nonce::zero();
        let nonce = self.nonce.get(who).unwrap_or(&binding);
        let new_nonce = nonce
            .checked_add(&T::Nonce::one())
            .ok_or(BlockError::OverflowIncrementNonce)?;
        self.nonce.insert(who.clone(), new_nonce);

        Ok(())
    }

    pub fn get_nonce(&self, who: &T::AccountId) -> T::Nonce {
        *self.nonce.get(who).unwrap_or(&T::Nonce::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestConfig;

    impl Config for TestConfig {
        type AccountId = String;
        type Nonce = u32;
        type BlockNumber = u128;
    }

    fn setup() -> Pallet<TestConfig> {
        Pallet::new()
    }

    #[test]
    fn init_system() {
        let system = setup();
        assert_eq!(system.block_number(), 0);
    }

    #[test]
    fn increment_block_number() {
        let mut system = setup();
        let _ = system.increment_block_number();
        assert_eq!(system.block_number(), 1);
    }

    #[test]
    fn increment_nonce() {
        let mut system = setup();
        let alice = String::from("Alice");
        let _ = system.increment_nonce(&alice);
        assert_eq!(system.get_nonce(&alice), 1);
    }
}
