use alloy_primitives::{Address, U64};
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
    pub fn get_balance(&self) -> U64 {
        self.balances.get(msg::sender()).to::<U64>()
    }

    pub fn inc_balance(&mut self) {
        let sender = msg::sender();
        let balance = self.balances.get(sender).to::<U64>();
        self.balances.insert(sender, balance + U64::from(1));
    }

    pub fn active() -> bool {
        T::ACTIVE
    }
}
