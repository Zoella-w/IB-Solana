use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum SocialInstruction {
    // 初始化账号
    InitializeUser { seed_type: String },
    // 关注
    FollowUser { user_to_follow: Pubkey },
    // 取消关注
    UnfollowUser { user_to_unfollow: Pubkey },
    // 查询关注
    QueryFollows,
    // 发帖
    PostContent { content: String },
    // 查询发帖
    QueryPosts,
}
