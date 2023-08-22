use rs_merkle::{Hasher, MerkleTree};
use ethers::utils::keccak256;

pub mod account_with_balance;
use account_with_balance::AccountWithBalance;

#[derive(Clone)]
struct Keccak256Algorithm {}

impl Hasher for Keccak256Algorithm {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        keccak256(data)
    }
}

fn order_accounts(accounts: &Vec<AccountWithBalance>) -> Vec<AccountWithBalance> {
  let mut accounts = accounts.clone();
  accounts.sort();
  accounts
}

fn create_merkle_tree(accounts: Vec<AccountWithBalance>) -> MerkleTree<Keccak256Algorithm> {
  let ordered_accounts = order_accounts(&accounts);
  let leaves: Vec<[u8; 32]> = ordered_accounts
      .iter()
      .map(|x| x.generate_hash())
      .collect();
  MerkleTree::<Keccak256Algorithm>::from_leaves(&leaves)
}

pub fn get_merkle_root(accounts: Vec<AccountWithBalance>) -> [u8; 32] {
  let ordered_accounts = order_accounts(&accounts);
  let merkle_tree = create_merkle_tree(ordered_accounts);
  merkle_tree.root().expect("Couldn't get the merkle root")
}

pub fn generate_proof_of_inclusion(accounts: Vec<AccountWithBalance>, account: AccountWithBalance) -> Vec<u8> {
  let ordered_accounts = order_accounts(&accounts);
  let merkle_tree = create_merkle_tree(ordered_accounts);
  let index = accounts
      .iter()
      .position(|x| x.packed() == account.packed())
      .expect("Couldn't find the index of the account");
  merkle_tree
  .proof(&[index])
  .to_bytes()
}

//pub fn generate_proof_of_absense(accounts: Vec<AccountWithBalance>, account: AccountWithBalance) -> Vec<u8> {}

#[cfg(test)]
mod tests {
    use super::*;

    use rs_merkle::MerkleProof;

    fn fixed_accounts() -> Vec<AccountWithBalance> {
      let mut accounts = [
        AccountWithBalance::new("F977814e90dA44bFA03b6295A0616a897441aceC", "1"),
        AccountWithBalance::new("47ac0Fb4F2D84898e4D9E7b4DaB3C24507a6D503", "2"),
        AccountWithBalance::new("A7A93fd0a276fc1C0197a5B5623eD117786eeD06", "3"),
        AccountWithBalance::new("cEe284F754E854890e311e3280b767F80797180d", "10"),
        AccountWithBalance::new("5754284f345afc66a98fbB0a0Afe71e0F007B949", "100")
      ].to_vec();
      accounts.sort();
      accounts
    }

    #[test]
    fn test_account_sorting() {
      let accounts = fixed_accounts();
      println!("{:?}", accounts);
      assert_eq!(accounts[0].address, "47ac0Fb4F2D84898e4D9E7b4DaB3C24507a6D503".parse().unwrap());
      assert_eq!(accounts[1].address, "5754284f345afc66a98fbB0a0Afe71e0F007B949".parse().unwrap());
      assert_eq!(accounts[2].address, "A7A93fd0a276fc1C0197a5B5623eD117786eeD06".parse().unwrap());
      assert_eq!(accounts[3].address, "cEe284F754E854890e311e3280b767F80797180d".parse().unwrap());
      assert_eq!(accounts[4].address, "F977814e90dA44bFA03b6295A0616a897441aceC".parse().unwrap());
    }

    #[test]
    fn test_create_merkle_tree() {
        let accounts = fixed_accounts();

        let merkle_tree = create_merkle_tree(accounts.clone());
        let root = merkle_tree.root().expect("Couldn't get the merkle root");

        assert_eq!(merkle_tree.depth(), 3);
        assert_eq!(merkle_tree.leaves_len(), 5);
        //Mannually calculated root hash
        assert_eq!(root, [
          0x62, 0xBC, 0x8B, 0xF4, 0xCB, 0x67, 0x25, 0x46, 
          0xF9, 0xE2, 0x5C, 0xF2, 0x0B, 0xAC, 0xFF, 0x9E, 
          0xAA, 0xE0, 0x47, 0x3A, 0x79, 0xA1, 0x68, 0x7D, 
          0x15, 0xF9, 0xC3, 0x26, 0x36, 0x74, 0x97, 0x32
        ]);
    }
        
    #[test]
    fn test_generate_proof_of_inclusion() {
        let accounts = fixed_accounts();
        let account = accounts[1].clone();
        let indices_to_prove = vec![1];

        let proof_bytes = generate_proof_of_inclusion(accounts.clone(), account.clone());

        let merkle_tree = create_merkle_tree(accounts.clone());
        let merkle_root = merkle_tree.root().expect("Couldn't get the merkle root");
        let leave_to_prove = account.generate_hash();

        let proof = MerkleProof::<Keccak256Algorithm>::try_from(proof_bytes).expect("couldn't parse proof");

        assert!(proof.verify(merkle_root, &indices_to_prove, &[leave_to_prove], accounts.len()));
    }

    #[test]
    fn test_generate_proof_of_inclusion_from_missing_account() {
      let accounts = fixed_accounts();
      let account = AccountWithBalance::new("0000000000000000000000000000000000000001", "1");
      let result = std::panic::catch_unwind(|| generate_proof_of_inclusion(accounts.clone(), account.clone()));
      assert!(result.is_err());
  }
}
