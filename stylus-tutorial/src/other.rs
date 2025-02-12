use alloy_primitives::Address;
use std::marker::PhantomData;
use stylus_sdk::{
    msg,
    prelude::*,
    storage::{StorageMap, StorageU64},
};

pub trait Params {
    const ACTIVE: bool;
    const LEN: u128;
}

#[storage]
pub struct Other<T> {
    balances: StorageMap<Address, StorageU64>,
    phanton: PhantomData<T>,
}

#[public]
impl<T: Params> Other<T> {
    pub fn get_balance(&self) -> u64 {
        self.balances.get(msg::sender()).to::<u64>()
    }

    pub fn active() -> bool {
        T::ACTIVE
    }
}
