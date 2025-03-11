mod balance;
mod support;
mod system;
mod types {
    use crate::support;

    pub type AccountId = String;
    pub type Balance = u128;
    pub type BlockNumber = u128;
    pub type Nonce = u32;
    pub type Extrinsic<'a> = support::Extrinsic<&'a AccountId, crate::RuntimeCall<'a>>;
    pub type Header = support::Header<BlockNumber>;
    pub type Block<'a> = support::Block<Header, Extrinsic<'a>>;
}

use std::error::Error;

use support::Dispatch;

pub enum RuntimeCall<'a> {
    BalanceTransfer {
        to: &'a types::AccountId,
        amount: types::Balance,
    },
}
impl system::Config for Runtime {
    type AccountId = types::AccountId;
    type Nonce = types::Nonce;
    type BlockNumber = types::BlockNumber;
}

impl balance::Config for Runtime {
    type Balance = types::Balance;
}

#[derive(Debug)]
pub struct Runtime {
    system: system::Pallet<Runtime>,
    balance: balance::Pallet<Runtime>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            system: system::Pallet::new(),
            balance: balance::Pallet::new(),
        }
    }

    fn execute_block(&mut self, block: types::Block) -> support::DispatchResult {
        self.system.increment_block_number()?;

        if self.system.block_number() != block.header.block_number {
            return Err(support::DispatchError::BlockNumberMismatch);
        }

        for (i, support::Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
            self.system.increment_nonce(&caller)?;
            let _ = self.dispatch(&caller, call).map_err(|e| {
                eprintln!(
                    "Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
                    block.header.block_number, i, e
                )
            });
        }

        Ok(())
    }
}

impl<'a> crate::support::Dispatch<'a> for Runtime {
    type Caller = &'a <Runtime as system::Config>::AccountId;
    type Call = RuntimeCall<'a>;
    // Dispatch a call on behalf of a caller. Increments the caller's nonce.
    //
    // Dispatch allows us to identify which underlying module call we want to execute.
    // Note that we extract the `caller` from the extrinsic, and use that information
    // to determine who we are executing the call on behalf of.
    fn dispatch(
        &mut self,
        caller: Self::Caller,
        runtime_call: Self::Call,
    ) -> support::DispatchResult {
        match runtime_call {
            RuntimeCall::BalanceTransfer { to, amount } => {
                self.balance.transfer(&caller, &to, amount)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut runtime = Runtime::new();

    let alice = String::from("Alice");
    let bob = String::from("Bob");
    let charlie = String::from("Charlie");

    runtime.balance.set_balance(&alice, 100);

    let block_1 = types::Block {
        header: support::Header { block_number: 1 },
        extrinsics: vec![
            support::Extrinsic {
                caller: &alice,
                call: RuntimeCall::BalanceTransfer {
                    to: &bob,
                    amount: 50,
                },
            },
            support::Extrinsic {
                caller: &alice,
                call: RuntimeCall::BalanceTransfer {
                    to: &charlie,
                    amount: 50,
                },
            },
        ],
    };

    runtime
        .execute_block(block_1)
        .map_err(|e| eprintln!("Error when executing block: {e}"))
        .expect("[ERROR] Failed to proccess block!");

    println!("{:?}", runtime);

    Ok(())
}
