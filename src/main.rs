use ethers::{
    abi::{encode_packed, Token},
    types::{Address, U256},
    utils::keccak256,
};
use rand::Rng;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
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
        .clone()
        .into_iter()
        .map(|account| account.generate_hash())
        .collect();
    // println!("{:?}", hashes);

    let root = generate_merkle_proof(hashes.clone());
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("{:?}", root);

    let proofs = generate_proof_of_inclusion(hashes, addresses[3]);
    println!("Proofs: {:?}", proofs.len());
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

// TODO: Ensure the tree is sorted here or change the name
fn generate_proof_of_inclusion(keys: Vec<[u8; 32]>, account: AccountWithBalance) -> Vec<[u8; 32]> {
    let mut proofs: Vec<[u8; 32]> = vec![];

    let mut target_node = account.generate_hash();
    proofs.push(target_node);

    let mut nodes = keys.clone();
    while nodes.len() > 1 {
        let mut level: Vec<[u8; 32]> = vec![];
        while let Some(left) = nodes.pop() {
            let right = nodes.pop().unwrap();
            let encoded =
                encode_packed(&[Token::Bytes(left.to_vec()), Token::Bytes(right.to_vec())])
                    .unwrap();
            level.push(keccak256(encoded.clone()));

            if nodes.len() == 1 {
                level.push(nodes.pop().unwrap());
            } else {
                // TODO: Am I missing the corners when an item is left alone when creating the
                // following layer?
                if left == target_node {
                    proofs.push(left);
                    target_node = keccak256(encoded.clone());
                } else if right == target_node {
                    proofs.push(right);
                    target_node = keccak256(encoded.clone());
                }
            }
        }

        nodes = level;
    }

    proofs
}
