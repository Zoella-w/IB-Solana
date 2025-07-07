use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::client;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

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

// 定义种子常量（用于生成 PDA）
const USER_PROFILE_SEED: &str = "profile"; // 用户档案种子
const USER_POST_SEED: &str = "post"; // 用户帖子种子

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

pub struct SocialClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
}

impl SocialClient {
    /// 创建新的社交客户端
    pub fn new(rpc_url: &str, program_id: Pubkey) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Self {
            rpc_client,
            program_id,
        }
    }

    /// 初始化用户账户（创建PDA）
    pub fn initialize_user(
        &self,
        user_keypair: &Keypair, // 用户密钥对（支付账户）
        seed_type: &str,        // 账户类型（"profile" 或 "post"）
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算 PDA 地址
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), seed_type.as_bytes()],
        );

        // 构建账户列表
        let accounts = vec![
            // 用户账户（支付者）
            AccountMeta::new(user_keypair.pubkey(), true),
            // PDA账户（将被创建）
            AccountMeta::new(pda, false),
            // 系统程序（用于创建账户）
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];

        // 创建初始化指令
        let initialize_user_instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::InitializeUser {
                seed_type: seed_type.to_string(),
            },
            accounts,
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![initialize_user_instruction])?;
        Ok(())
    }

    /// 关注其他用户
    pub fn follow_user(
        &self,
        user_keypair: &Keypair, // 当前用户密钥对
        follow_user: Pubkey,    // 要关注的用户公钥
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算当前用户的 PDA 地址
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_bytes()],
        );

        // 创建关注指令
        let follow_user_instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::FollowUser {
                user_to_follow: follow_user,
            },
            // 只需要操作 PDA 账户
            vec![AccountMeta::new(pda, false)],
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![follow_user_instruction])?;
        Ok(())
    }

    /// 查询关注列表
    pub fn query_followers(
        &self,
        user_keypair: &Keypair, // 当前用户密钥对
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算当前用户的 PDA 地址
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_bytes()],
        );

        // 创建查询指令
        let query_followers_instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::QueryFollows,
            // 只需要读取 PDA 账户
            vec![AccountMeta::new(pda, false)],
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![query_followers_instruction])?;
        Ok(())
    }

    /// 取消关注
    pub fn unfollow_user(
        &self,
        user_keypair: &Keypair, // 当前用户密钥对
        unfollow_user: Pubkey,  // 要取消关注的用户公钥
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算当前用户的 PDA 地址
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_bytes()],
        );

        // 创建查询指令
        let unfollow_user_instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::UnfollowUser {
                user_to_unfollow: unfollow_user,
            },
            // 只需要读取 PDA 账户
            vec![AccountMeta::new(pda, false)],
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![unfollow_user_instruction])?;
        Ok(())
    }

    /// 发布帖子
    pub fn post_content(
        &self,
        user_keypair: &Keypair, // 用户密钥对（支付账户）
        content: String,        // 帖子内容
        id: u64,                // 帖子ID（唯一标识）
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算用户帖子计数 PDA
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_POST_SEED.as_bytes()],
        );

        // 计算特定帖子的 PDA
        let post_pda = get_pda(
            &self.program_id,
            &[
                user_keypair.pubkey().as_ref(), // 用户公钥
                USER_POST_SEED.as_bytes(),      // 帖子种子
                &[id as u8],                    // 帖子ID（转换为字节）
            ],
        );

        // 构建发布指令
        let post_content_instruction = Instruction::new_with_borsh(
            self.program_id,                                      // 当前程序ID
            &SocialInstruction::PostContent { content: content }, // 指令数据
            // 账户列表
            vec![
                // 用户账户（支付者，需要签名）
                AccountMeta::new(user_keypair.pubkey(), true),
                // 用户帖子计数 PDA（可写）
                AccountMeta::new(pda, false),
                // 帖子内容 PDA（可写）
                AccountMeta::new(post_pda, false),
                // 系统程序（只读，用于账户创建）
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![post_content_instruction]);

        Ok(())
    }

    /// 查询帖子
    pub fn query_posts(
        &self,
        user_keypair: &Keypair, // 用户密钥对
        id: u64,                // 要查询的帖子ID
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 计算用户帖子计数 PDA
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_POST_SEED.as_bytes()],
        );

        // 计算特定帖子的 PDA
        let post_pda = get_pda(
            &self.program_id,
            &[
                user_keypair.pubkey().as_ref(), // 用户公钥
                USER_POST_SEED.as_bytes(),      // 帖子种子
                &[id as u8],                    // 帖子ID（转换为字节）
            ],
        );

        // 构建查询指令
        let query_post_instruction = Instruction::new_with_borsh(
            self.program_id,                // 当前程序ID
            &SocialInstruction::QueryPosts, // 指令数据
            // 账户列表（只需读取）
            vec![
                // 用户帖子计数 PDA（可读）
                AccountMeta::new(pda, false),
                // 帖子内容 PDA（可读）
                AccountMeta::new(post_pda, false),
            ],
        );

        // 发送交易
        self.send_instruction(user_keypair, vec![query_post_instruction]);

        Ok(())
    }

    // 内部方法：发送指令并确认交易
    fn send_instruction(
        &self,
        payer: &Keypair,                // 支付账户
        instructions: Vec<Instruction>, // 要执行的指令列表
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取最新区块哈希（防止重放攻击）
        let latest_blockhash = self.rpc_client.get_latest_blockhash()?;

        // 创建并签名交易
        let transaction = Transaction::new_signed_with_payer(
            &instructions,         // 指令列表
            Some(&payer.pubkey()), // 支付账户
            &[payer],              // 签名者列表
            latest_blockhash,      // 区块哈希
        );

        // 发送并确认交易
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("✅ 交易成功: {}", signature);

        Ok(())
    }
}

// 计算 PDA 地址
fn get_pda(program_id: &Pubkey, seed: &[&[u8]]) -> Pubkey {
    // 使用程序 ID 和 种子生成 PDA
    let (pda, _bump) = Pubkey::find_program_address(seed, &program_id);
    println!("🆔 计算PDA: {:?}", pda);
    pda
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let user_profile = UserProfile::new();
    // println!(
    //     "user_profile len is {:?}",
    //     borsh::to_vec(&user_profile).unwrap().len()
    // ); // user_profile len is 6

    // let post_profile = UserPost::new();
    // println!(
    //     "post_profile len is {:?}",
    //     borsh::to_vec(&post_profile).unwrap().len()
    // ); // post_profile len is 8

    // 设置程序 ID（实际部署时替换为真实 ID）
    let program_id = Pubkey::from_str_const("4C8vNaH53MvQzm9q47Wz8TPDf97iXf7bTr6SaortLbMX");
    // 从文件加载用户密钥对
    let user_keypair = read_keypair_file("/Users/wangyueying/.config/solana/ad2.json")?;

    // 创建社交客户端（连接本地测试节点）
    let client = SocialClient::new("http://127.0.0.1:8899", program_id);

    // // === UserProfile 使用示例 ===

    // // 1. 初始化 user_profile 账户
    // client.initialize_user(&user_keypair, USER_PROFILE_SEED)?;

    // // 2. 关注用户
    // let follow_user = Pubkey::from_str_const("Gx46VGxLxqwH8FSP7E3EAXQvp6FNGyouTQMuT2aD4etx");
    // client.follow_user(&user_keypair, follow_user)?;

    // // 3. 查询账户信息（关注数据）
    // client.query_followers(&user_keypair)?;

    // // 4. 取消关注
    // let unfollow_user = Pubkey::from_str_const("Gx46VGxLxqwH8FSP7E3EAXQvp6FNGyouTQMuT2aD4etx");
    // client.unfollow_user(&user_keypair, unfollow_user)?;

    // // 5. 查询账户信息（取消关注后的关注数据）
    // client.query_followers(&user_keypair)?;

    // // === UserPost 使用示例 ===

    // 1. 创建 UserPost 账户
    client.initialize_user(&user_keypair, USER_POST_SEED)?;

    // 2. 发送帖子1
    let mut content = "hello solana, id: 1".to_string();
    let mut id = 1;
    client.post_content(&user_keypair, content, id)?;

    // 3. 查询帖子1
    client.query_posts(&user_keypair, id)?;

    // 4. 发送帖子2
    content = "hello solana, id: 2".to_string();
    id = 2;
    client.post_content(&user_keypair, content, id)?;

    // 5. 查询帖子2
    client.query_posts(&user_keypair, id)?;

    Ok(())
}
