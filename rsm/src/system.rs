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

#[derive(Debug)]
pub struct Pallet<AccountId, BlockNumber, Nonce> {
    bloc_number: BlockNumber,
    nonce: BTreeMap<AccountId, Nonce>, // Numver of transactions someone make on the blockchain
}

impl<AccountId, BlockNumber, Nonce> Pallet<AccountId, BlockNumber, Nonce>
where
    AccountId: Ord + Clone,
    BlockNumber: Zero + One + CheckedSub + CheckedAdd + Copy,
    Nonce: Zero + One + CheckedSub + CheckedAdd + Copy,
{
    pub fn new() -> Self {
        Self {
            bloc_number: BlockNumber::zero(),
            nonce: BTreeMap::new(),
        }
    }

    pub fn block_number(&self) -> BlockNumber {
        self.bloc_number
    }

    pub fn increment_block_number(&mut self) -> Result<(), BlockError> {
        self.bloc_number = self
            .bloc_number
            .checked_add(&BlockNumber::one())
            .ok_or(BlockError::OverflowIncrementBlockNumber)?;

        Ok(())
    }

    pub fn increment_nonce(&mut self, who: &AccountId) -> Result<(), BlockError> {
        let binding = Nonce::zero();
        let nonce = self.nonce.get(who).unwrap_or(&binding);
        let new_nonce = nonce
            .checked_add(&Nonce::one())
            .ok_or(BlockError::OverflowIncrementNonce)?;
        self.nonce.insert(who.clone(), new_nonce);

        Ok(())
    }

    pub fn get_nonce(&self, who: &AccountId) -> Nonce {
        *self.nonce.get(who).unwrap_or(&Nonce::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;

    fn setup() -> Pallet<types::AccountId, types::BlockNumber, types::Nonce> {
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
