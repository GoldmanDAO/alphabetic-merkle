use criterion::{criterion_group, criterion_main, Criterion};

use ethers::types::{Address, U256};
use rand::Rng;

use merkletree::merkle_tree::get_merkle_root;
use merkletree::account_with_balance::AccountWithBalance;

fn bench_merkle(c: &mut Criterion) {
  let mut rnd_addresses: Vec<AccountWithBalance> = vec![];
  let mut rng = rand::thread_rng();

  for _ in 0..1000000 {
      let random_number: f64 = rng.gen();
      rnd_addresses.push(AccountWithBalance {
          address: Address::random(),
          balance: U256::from_little_endian(&random_number.to_le_bytes()),
      });
  }

  c.bench_function("Test 100000", |b| b.iter(|| get_merkle_root(&rnd_addresses[0..100000].to_vec())));
  c.bench_function("Test 1000000", |b| b.iter(|| get_merkle_root(&rnd_addresses[0..1000000].to_vec())));
}

criterion_group!(benches, bench_merkle);
criterion_main!(benches);