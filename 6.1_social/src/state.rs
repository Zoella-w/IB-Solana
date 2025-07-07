use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UserProfile {
    pub data_len: u16,
    pub follows: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UserPost {
    pub post_count: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Post {
    pub content: String,
    pub timestamp: u64,
}

impl UserProfile {
    pub fn new() -> Self {
        Self {
            data_len: 0,
            follows: Vec::new(),
        }
    }

    pub fn follow(&mut self, user: Pubkey) {
        self.follows.push(user);
        self.data_len = self.follows.len() as u16;
    }

    pub fn unfollow(&mut self, user_to_unfollow: Pubkey) {
        self.follows.retain(|&x| x != user_to_unfollow);
        self.data_len = self.follows.len() as u16;
    }
}

impl UserPost {
    pub fn new() -> Self {
        Self { post_count: 0 }
    }

    pub fn add_post(&mut self) {
        self.post_count += 1;
    }

    pub fn get_count(&self) -> u64 {
        self.post_count
    }
}

impl Post {
    pub fn new(content: String, timestamp: u64) -> Self {
        Self { content, timestamp }
    }
}
