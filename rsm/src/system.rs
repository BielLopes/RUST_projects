use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Overflow when incrementing block number")]
    OverflowIncrementBlockNumber,
    #[error("Overflow when incrementing nonce")]
    OverflowIncrementNonce,
}

pub struct Pallet {
    bloc_number: u128,
    nonce: BTreeMap<String, u32>, // Numver of transactions someone make on the blockchain
}

impl Pallet {
    pub fn new() -> Self {
        Self {
            bloc_number: 0,
            nonce: BTreeMap::new(),
        }
    }

    pub fn block_number(&self) -> u128 {
        self.bloc_number
    }

    pub fn increment_block_number(&mut self) -> Result<(), BlockError> {
        self.bloc_number = self
            .bloc_number
            .checked_add(1)
            .ok_or(BlockError::OverflowIncrementBlockNumber)?;

        Ok(())
    }

    pub fn increment_nonce(&mut self, who: &String) -> Result<(), BlockError> {
        let nonce = self.nonce.get(who).unwrap_or(&0);
        let new_nonce = nonce
            .checked_add(1)
            .ok_or(BlockError::OverflowIncrementNonce)?;
        self.nonce.insert(who.clone(), new_nonce);

        Ok(())
    }

    pub fn get_nonce(&self, who: &String) -> u32 {
        *self.nonce.get(who).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Pallet {
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
