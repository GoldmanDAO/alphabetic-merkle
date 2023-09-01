use ethers::{
    abi::{encode_packed, Token},
    types::{Address, U256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AccountWithBalance {
    pub address: Address,
    pub balance: U256,
}

impl AccountWithBalance {
    pub fn new(address: &str, balance: &str) -> Self {
        Self {
            address: address.to_lowercase().parse().unwrap(),
            balance: U256::from_dec_str(balance).unwrap(),
        }
    }

    pub fn packed(&self) -> Vec<u8> {
        encode_packed(&[Token::Address(self.address), Token::Uint(self.balance)]).unwrap()
    }

    pub fn generate_hash(&self) -> [u8; 32] {
        keccak256(self.packed())
    }
}

impl Ord for AccountWithBalance {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.packed().cmp(&other.packed())
    }
}

impl PartialOrd for AccountWithBalance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AccountWithBalance {
    fn eq(&self, other: &Self) -> bool {
        self.packed() == other.packed()
    }
}

impl Eq for AccountWithBalance {}
