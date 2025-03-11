mod balance;
mod system;
mod types {
    pub type AccountId = String;
    pub type Balance = u128;
    pub type BlockNumber = u128;
    pub type Nonce = u32;
}

impl balance::Config for Runtime {
    type AccountId = types::AccountId;
    type Balance = types::Balance;
}

impl system::Config for Runtime {
    type AccountId = types::AccountId;
    type Nonce = types::Nonce;
    type BlockNumber = types::BlockNumber;
}

#[derive(Debug)]
pub struct Runtime {
    balance: balance::Pallet<Runtime>,
    system: system::Pallet<Runtime>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            balance: balance::Pallet::new(),
            system: system::Pallet::new(),
        }
    }
}

fn main() {
    let mut runtime = Runtime::new();

    let alice = String::from("Alice");
    let bob = String::from("Bob");

    runtime.balance.set_balance(&alice, 100);

    runtime.system.increment_block_number().unwrap();
    assert_eq!(runtime.system.block_number(), 1);

    let _ = runtime.system.increment_nonce(&alice);

    let _ = runtime.balance.transfer(&alice, &bob, 50).map_err(|err| {
        eprintln!("Error on tranfeerence: {err}");
    });

    println!("{:?}", runtime);
}
