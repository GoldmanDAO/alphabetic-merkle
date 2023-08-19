use ethers::{
    abi::{encode_packed, Token},
    types::{Address, U256},
    utils::keccak256,
};
use rand::Rng;
use std::time::Instant;

#[derive(Debug)]
struct AccountWithBalance {
    account: Address,
    balance: U256,
}

impl AccountWithBalance {
    fn generate_hash(&self) -> [u8; 32] {
        keccak256(
            encode_packed(&[Token::Address(self.account), Token::Uint(self.balance)]).unwrap(),
        )
    }
}

fn main() {
    let mut addresses = generate_addresses();
    addresses.sort_by(|a, b| a.account.0.cmp(&b.account.0));
    println!("Starting now");
    let now = Instant::now();

    let hashes: Vec<[u8; 32]> = addresses
        .into_iter()
        .map(|account| account.generate_hash())
        .collect();
    // println!("{:?}", hashes);

    let root = generate_merkle_proof(hashes);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("{:?}", root);
}

fn generate_addresses() -> Vec<AccountWithBalance> {
    let mut rnd_addresses: Vec<AccountWithBalance> = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..100000 {
        let random_number: f64 = rng.gen();
        rnd_addresses.push(AccountWithBalance {
            account: Address::random(),
            balance: U256::from_little_endian(&random_number.to_le_bytes()),
        });
    }

    rnd_addresses
}

fn generate_merkle_proof(keys: Vec<[u8; 32]>) -> [u8; 32] {
    let mut nodes = keys.clone();

    while nodes.len() > 1 {
        let mut level: Vec<[u8; 32]> = vec![];
        while let Some(left) = nodes.pop() {
            let right = nodes.pop().unwrap();
            let encoded =
                encode_packed(&[Token::Bytes(left.to_vec()), Token::Bytes(right.to_vec())]);
            level.push(keccak256(encoded.unwrap()));

            if nodes.len() == 1 {
                level.push(nodes.pop().unwrap());
            }
        }

        nodes = level;
    }

    nodes[0]
}
