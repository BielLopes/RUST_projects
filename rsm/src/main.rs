mod balance;
mod proof_of_existence;
mod support;
mod system;
mod types {
    use crate::support;

    pub type AccountId = String;
    pub type Balance = u128;
    pub type BlockNumber = u128;
    pub type Nonce = u32;
    pub type Extrinsic = support::Extrinsic<AccountId, crate::RuntimeCall>;
    pub type Header = support::Header<BlockNumber>;
    pub type Block = support::Block<Header, Extrinsic>;
    pub type Content = String;
}

use std::error::Error;

use support::Dispatch;

impl system::Config for Runtime {
    type AccountId = types::AccountId;
    type Nonce = types::Nonce;
    type BlockNumber = types::BlockNumber;
}

impl balance::Config for Runtime {
    type Balance = types::Balance;
}

impl proof_of_existence::Config for Runtime {
    type Content = types::Content;
}

#[derive(Debug)]
#[macros::runtime]
pub struct Runtime {
    system: system::Pallet<Runtime>,
    balance: balance::Pallet<Runtime>,
    proof_of_existence: proof_of_existence::Pallet<Runtime>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut runtime = Runtime::new();

    let alice = String::from("Alice");
    let bob = String::from("Bob");
    let charlie = String::from("Charlie");

    runtime.balance.set_balance(&&alice, 100);

    let block_1 = types::Block {
        header: support::Header { block_number: 1 },
        extrinsics: vec![
            support::Extrinsic {
                caller: alice.clone(),
                call: RuntimeCall::balance(balance::Call::transfer {
                    to: bob.clone(),
                    amount: 50,
                }),
            },
            support::Extrinsic {
                caller: alice.clone(),
                call: RuntimeCall::balance(balance::Call::transfer {
                    to: charlie,
                    amount: 40,
                }),
            },
        ],
    };

    runtime
        .execute_block(block_1)
        .map_err(|e| eprintln!("Error when executing block: {e}"))
        .expect("[ERROR] Failed to proccess block!");

    let block_2 = types::Block {
        header: support::Header { block_number: 2 },
        extrinsics: vec![support::Extrinsic {
            caller: alice,
            call: RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
                claim: String::from("Asset"),
            }),
        }],
    };

    runtime
        .execute_block(block_2)
        .map_err(|e| eprintln!("Error when executing block: {e}"))
        .expect("[ERROR] Failed to proccess block!");

    println!("{:?}", runtime);

    Ok(())
}
