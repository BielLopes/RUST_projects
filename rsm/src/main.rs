mod balance;
mod system;

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
    runtime.balance.set_balance(&bob, 100);

    runtime.system.increment_block_number().unwrap();
    runtime.system.increment_nonce(&alice).unwrap();

    runtime.balance.transfer(&alice, &bob, 50).unwrap();

    println!("Alice balance: {}", runtime.balance.balance(&alice));
    println!("Bob balance: {}", runtime.balance.balance(&bob));
    println!("Block number: {}", runtime.system.block_number());
    println!("Alice nonce: {}", runtime.system.get_nonce(&alice));
}
