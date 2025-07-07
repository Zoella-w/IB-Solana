use crate::instruction::*;
use crate::state::*;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::borsh1::try_from_slice_unchecked;
use solana_program::clock::Clock;
use solana_program::lamports;
use solana_program::system_instruction;
// Borsh 反序列化库
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{Sysvar, rent::Rent},
};

// 常量定义
const PUBKEY_SIZE: usize = 32; // 公钥长度（32字节）
const U16_SIZE: usize = 2; // u16类型长度（2字节）—— 关注数量为 u16类型
const USER_PROFILE_SIZE: usize = 6; // 用户档案基础大小（6字节）
const MAX_FOLLOWER_COUNT: usize = 200; // 最大关注人数限制
const USER_POST_SIZE: usize = 8; // 用户帖子基础大小（8字节）

pub struct Processor;

impl Processor {
    /// 主入口函数 - 处理所有指令
    pub fn process_instruction(
        program_id: &Pubkey,      // 当前程序 ID
        accounts: &[AccountInfo], // 传入的账户列表
        instruction_data: &[u8],  // 指令数据
    ) -> ProgramResult {
        // 反序列化指令数据
        let instruction = SocialInstruction::try_from_slice(instruction_data)?;

        // 根据指令类型分发处理
        match instruction {
            SocialInstruction::InitializeUser { seed_type } => {
                // 初始化用户账户
                Self::initialize_user(program_id, accounts, seed_type)
            }
            SocialInstruction::FollowUser { user_to_follow } => {
                // 关注其他用户
                Self::follow_user(accounts, user_to_follow)
            }
            SocialInstruction::QueryFollows => {
                // 查询关注列表
                Self::query_follows(accounts)
            }
            SocialInstruction::UnfollowUser { user_to_unfollow } => {
                // 取消关注
                Self::unfollow_user(accounts, user_to_unfollow)
            }
            SocialInstruction::PostContent { content } => {
                // 发布帖子
                Self::post_content(program_id, accounts, content)
            }
            SocialInstruction::QueryPosts => {
                // 查询帖子
                Self::query_posts(accounts)
            }
        }
    }

    /// 初始化用户账户（创建PDA账户）
    fn initialize_user(
        program_id: &Pubkey,      // 当前程序 ID
        accounts: &[AccountInfo], // 账户列表
        seed_type: String,        // 种子类型（"profile" 或 "post"）
    ) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();

        // 解析账户：
        // 1. 用户账户（支付账户）
        let user_account = next_account_info(account_info_iter)?;
        // 2. PDA 账户（将创建的账户）
        // - PDA 账户作为​​用户专属的存储空间​​，用于保存用户的社交数据（如关注列表）
        // - 每个用户有一个对应的 PDA 账户，其地址由用户公钥和固定种子（"profile" 或 "post"）派生
        // - PDA 账户的所有者是程序本身，保证只有程序能修改数据，外部账户无法直接操作 PDA 账户数据
        let pda_account = next_account_info(account_info_iter)?;
        // 3. 系统程序账户
        let system_program = next_account_info(account_info_iter)?;

        // 根据种子类型确定种子值
        let seed = match seed_type.as_str() {
            "profile" => "profile",                         // 用户档案种子
            "post" => "post",                               // 用户帖子种子
            _ => return Err(ProgramError::InvalidArgument), // 无效种子类型
        };
        msg!("seed: {:?}", seed); // 日志输出种子值

        // 计算 PDA 地址 和 bump seed
        // bump seed 是一个 1 字节的数字（0-255）​​，用于在生成 PDA 地址时进行微调，确保生成的地址​​不在椭圆曲线上​​
        // - 无密钥签名​：程序可以用它证明对 PDA 的控制权
        // - ​​地址确定性​​：相同输入总是生成相同 PDA
        // - ​​安全验证​​：防止地址冲突
        let (pda, bump_seed) = Pubkey::find_program_address(
            &[user_account.key.as_ref(), seed.as_bytes()], // 种子：用户公钥 + 种子类型
            program_id,
        );
        msg!("pda: {:?}", pda); // 日志输出 PDA 地址

        // 验证传入的 PDA 账户是否匹配计算出的 PDA
        // - pda：程序根据输入参数计算出的 PDA 地址
        // - pda_account.key：​客户端传入​的 PDA 账户地址
        if pda != pda_account.key.clone() {
            return Err(ProgramError::InvalidArgument);
        }

        // 获取租金系统变量
        let rent = Rent::get()?;

        // 根据账户类型计算所需空间
        let space = match seed_type.as_str() {
            "profile" => compute_profile_space(MAX_FOLLOWER_COUNT), // 用户档案空间
            "post" => USER_POST_SIZE,                               // 帖子账户空间
            _ => return Err(ProgramError::InvalidArgument),
        };

        // 计算所需租金（lamports）
        let lamports = rent.minimum_balance(space);

        // 创建账户指令
        let create_account_ix = system_instruction::create_account(
            user_account.key, // 支付账户
            &pda,             // 新账户地址
            lamports,         // 租金金额
            space as u64,     // 账户空间
            program_id,       // 所有者程序
        );

        // 执行带签名的跨程序调用（创建账户）
        invoke_signed(
            &create_account_ix, // 创建账户指令
            // 涉及的账户
            &[
                user_account.clone(),   // 支付账户
                pda_account.clone(),    // 目标账户：新建的 pda 账户
                system_program.clone(), // 系统程序：只有系统程序有权创建新账户
            ],
            // 签名种子
            // - 用户公钥：作用是身份绑定，防止越权访问
            // - 种子类型：作用是功能隔离，防止数据污染
            // - bump seed：作用是抗冲突，防止地址碰撞攻击
            &[&[
                user_account.key.as_ref(), // 用户公钥
                seed.as_bytes(),           // 种子类型
                &[bump_seed],              // bump seed
            ]],
        )?;

        // 根据账户类型初始化数据
        match seed_type.as_str() {
            "profile" => {
                // 创建新的用户档案
                let user_profile = UserProfile::new();
                // 序列化数据到 PDA 账户
                // try_borrow_mut_data 安全获取账户数据的​​可变引用（相比 borrow_mut_data 更安全，不会 panic）
                user_profile.serialize(&mut *pda_account.try_borrow_mut_data()?)?;
            }
            "post" => {
                // 帖子账户初始化
                let user_post = UserPost::new();
                // 序列化数据到 PDA 账户
                // try_borrow_mut_data 安全获取账户数据的​​可变引用（相比 borrow_mut_data 更安全，不会 panic）
                user_post.serialize(&mut *pda_account.try_borrow_mut_data()?)?;
            }
            _ => return Err(ProgramError::InvalidArgument),
        };

        Ok(())
    }

    /// 关注其他用户
    fn follow_user(
        accounts: &[AccountInfo],
        user_to_follow: Pubkey, // 要关注的用户公钥
    ) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        // 获取 PDA 账户（用户档案）
        let pda_account = next_account_info(account_info_iter)?;

        // 计算账户数据大小
        let mut size: usize = 0;

        // 只读作用域（不能同时存在可变和不可变借用）
        {
            // 借用账户数据（只读）
            let data = &pda_account.data.borrow();

            // 解析关注数量
            // 「关注数量」存储在数据的前2字节
            let len = &data[..U16_SIZE]; // 前2个元素的数组切片
            let pubkey_count = bytes_to_u16(len).unwrap(); // 将关注数量转换为 u16 类型

            // 计算完整数据大小
            size = compute_profile_space(pubkey_count as usize);
            msg!("size is {:?}", size); // 日志输出大小
        }
        // 释放只读借用

        // 反序列化用户档案数据
        let mut user_profile = UserProfile::try_from_slice(
            &pda_account.data.borrow()[..size], // 只读取有效数据部分
        )?;
        msg!("user_profile is {:?}", user_profile); // 日志输出用户档案

        // 执行关注操作
        user_profile.follow(user_to_follow);

        // 将序列化更新后的数据返回给 pda 账户
        // 获取可写借用
        user_profile.serialize(&mut *pda_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// 查询关注列表
    fn query_follows(accounts: &[AccountInfo]) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        // 获取 PDA 账户（用户档案）
        let pda_account = next_account_info(account_info_iter)?;

        // 反序列化用户档案数据（使用不安全但高效的方法）
        let user_profile =
            try_from_slice_unchecked::<UserProfile>(&pda_account.data.borrow()).unwrap();

        // 日志输出用户档案（包含关注列表）
        msg!("user_profile is {:?}", user_profile);

        Ok(())
    }

    // 取消关注用户
    fn unfollow_user(accounts: &[AccountInfo], user_to_unfollow: Pubkey) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        // 获取 PDA 账户（用户档案）
        let pda_account = next_account_info(account_info_iter)?;

        // 创建新的用户档案
        // 反序列化用户档案数据（使用不安全但高效的方法）
        let mut user_profile = try_from_slice_unchecked::<UserProfile>(&pda_account.data.borrow())?;
        user_profile.unfollow(user_to_unfollow);

        // 序列化数据到 PDA 账户
        // try_borrow_mut_data 安全获取账户数据的​​可变引用（相比 borrow_mut_data 更安全，不会 panic）
        user_profile.serialize(&mut *pda_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// 发布内容
    fn post_content(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        content: String,
    ) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?; // 用户账户（支付账户）
        let pda_account = next_account_info(account_info_iter)?; // 用户帖子计数账户（PDA）
        let post_pda_account = next_account_info(account_info_iter)?; // 新帖子内容账户（将创建）
        let system_program = next_account_info(account_info_iter)?; // 系统程序账户

        // 获取当前时间戳
        let clock = Clock::get()?; // 获取区块链时钟
        let timestamp = clock.unix_timestamp as u64; // 转换为 u64 时间戳

        // 更新用户帖子计数
        // 反序列化用户帖子计数账户
        let mut user_post = try_from_slice_unchecked::<UserPost>(&pda_account.data.borrow())?;

        // 增加帖子计数（相当于生成新帖子 ID）
        user_post.add_post();

        // 将更新后的计数写回账户
        user_post.serialize(&mut *pda_account.try_borrow_mut_data()?)?;

        // 获取最新帖子ID
        let count = user_post.get_count(); // 当前帖子总数（作为新帖子的ID）

        // 计算新帖子的 PDA 地址
        let (pda, bump_seed) = Pubkey::find_program_address(
            &[
                user_account.key.as_ref(), // 用户公钥（确保帖子归属）
                "post".as_bytes(),         // 固定种子（标识帖子类型）
                &[count as u8],            // 动态种子（帖子ID，确保唯一性）
            ],
            program_id,
        );

        // 创建帖子数据结构
        let post = Post::new(content, timestamp); // 包含内容和时间戳

        // 计算新账户所需资源
        let rent = Rent::get()?; // 获取租金信息
        let space = borsh::to_vec(&post).unwrap().len(); // 计算帖子数据大小
        let lamports = rent.minimum_balance(space); // 计算所需租金

        // 创建账户指令
        let post_content_ix = system_instruction::create_account(
            user_account.key, // 支付账户
            &pda,             // 新账户地址（帖子PDA）
            lamports,         // 租金金额
            space as u64,     // 账户空间大小
            program_id,       // 所有者程序（当前程序）
        );

        // 执行带签名的跨程序调用（创建账户）
        invoke_signed(
            &post_content_ix, // 创建账户指令
            &[
                user_account.clone(),     // 支付账户
                post_pda_account.clone(), // 新帖子账户
                system_program.clone(),   // 系统程序
            ],
            // 签名种子（用于 PDA 签名）
            &[&[
                user_account.key.as_ref(), // 用户公钥（身份绑定）
                "post".as_bytes(),         // 种子类型（功能隔离）
                &[count as u8],            // 帖子ID（唯一标识）
                &[bump_seed],              // bump seed（确保 PDA 有效）
            ]],
        )?;

        // 将帖子数据写入新账户
        post.serialize(&mut *post_pda_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// 查询帖子
    fn query_posts(accounts: &[AccountInfo]) -> ProgramResult {
        // 获取账户迭代器
        let account_info_iter = &mut accounts.iter();
        let pda_account = next_account_info(account_info_iter)?; // 用户帖子计数账户
        let pda_post_account = next_account_info(account_info_iter)?; // 帖子内容账户

        // 反序列化用户帖子数据（使用不安全但高效的方法）
        let user_post = try_from_slice_unchecked::<UserPost>(&pda_account.data.borrow())?;

        // 日志输出用户帖子数据（帖子数量）
        msg!("user_post is {:?}", user_post);

        // 反序列化帖子内容（使用不安全但高效的方法）
        let post = try_from_slice_unchecked::<Post>(&pda_post_account.data.borrow())?;

        // 日志输出帖子内容和时间戳
        msg!("post: {:?} at {:?}", post.content, post.timestamp);

        Ok(())
    }
}

// 计算用户账户空间
fn compute_profile_space(pubkey_count: usize) -> usize {
    // 新 UserProfile 的大小 + 存储的公钥数量 * 公钥大小
    return USER_PROFILE_SIZE + pubkey_count * PUBKEY_SIZE;
}

// 假设 bytes 为 [0x10, 0x27]
// 转换的字节值为 0x27 0x10
// 结果是 0*16^0 + 1*16^1 + 7*16^2 + 2*16^3 = 10000
fn bytes_to_u16(bytes: &[u8]) -> Option<u16> {
    if bytes.len() != 2 {
        return None;
    }

    let mut array = [0u8; 2]; // 创建一个大小为2的u8数组​​​：[0, 0]
    array.copy_from_slice(bytes); // 数据复制：比如 [0x10, 0x27]
    Some(u16::from_le_bytes(array)) // 字节序转换
}
