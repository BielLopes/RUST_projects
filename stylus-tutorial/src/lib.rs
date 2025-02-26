// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::Address;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    msg,
    prelude::*,
    storage::{StorageAddress, StorageString},
};

use other::{Other, Params};

mod other;

pub struct MyParams;
impl Params for MyParams {
    const ACTIVE: bool = true;
    const LEN: u128 = 42;
}

// sol_interface! {
//     interface IDex {
//         function active() external pure returns (bool);
//     }
// }

#[storage]
pub struct Owner {
    owner_adr: StorageAddress,
    username: StorageString,
}

sol_storage! {
    #[entrypoint]
    pub struct Contract {
        Owner owner;
        #[borrow]
        Other<MyParams> other;
    }
}

impl Contract {
    // Method defined here will not be exposed to the people using the contract
}

#[public]
#[inherit(Other<MyParams>)]
impl Contract {
    // Method defined here will be exposed to the people using the contract
    // #[payable] is used to indicate that the method can receive funds
    // #[pure] is used to indicate that the method does not modify the state of the contract, that
    // means, can't have the &self as parameter
    pub fn get_owner(&self) -> (Address, String) {
        (self.owner.owner_adr.get(), self.owner.username.get_string())
    }

    pub fn give_ownership(&mut self, new_owner: Address, new_username: String) -> String {
        match self.owner.owner_adr.get() {
            addr if (addr == Address::default() || addr == msg::sender()) => {
                self.owner.owner_adr.set(new_owner);
                self.owner.username.set_str(new_username.clone());
                format!("Ownership transferred! Congratulations {}!", new_username)
            }
            _ => String::from("You are not the owner of this contract!"),
        }
    }

    // pub fn example(&self, dex: IDex) -> Result<(), Vec<u8>> {
    //     dex.active(self)?;
    //     Ok(())
    // }
}
