use borsh::{BorshDeserialize, BorshSerialize};

// 为 TokenInstruction 实现 BorshDeserialize 和 BorshSerialize trait
#[derive(BorshDeserialize, BorshSerialize)]
pub enum TokenInstruction {
    CreateToken { decimals: u8 },
    Mint { amount: u64 },
}
