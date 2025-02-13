mod balance;
mod system;

#[derive(Debug)]
pub struct Runtime {
    balance: balance::Pallet,
    system: system::Pallet,
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
