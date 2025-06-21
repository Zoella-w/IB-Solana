use anchor_lang::prelude::*;

declare_id!("9mcnWsJdUp8NMtLkNbNDKG8g6QEzTq5ZUrHYA4gLcxXm");

#[program]
pub mod anchor_todo {
    use super::*;

    // #[account] 是 Anchor 框架的一个属性宏，用于定义程序的账户数据结构
    #[account]
    pub struct UserAccount {
        pub author: Pubkey,
        pub todos: Vec<String>, // 待办事项列表
    }

    // 初始化用户账户，创建一个新的用户账户
    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        // 设置账户所有者为当前调用者
        user_account.author = *ctx.accounts.author.key;
        Ok(())
    }

    // 添加待办事项
    pub fn add_todo(ctx: Context<ManageTodo>, content: String) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        // 将新待办事项添加到用户账户的todos列表末尾
        user_account.todos.push(content);
        Ok(())
    }

    // 删除待办事项
    pub fn remove_todo(ctx: Context<ManageTodo>, index: u8) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        // 检查索引是否有效（不超出范围）
        if index as usize >= user_account.todos.len() {
            return Err(ProgramError::InvalidArgument.into());
        }
        // 删除指定位置的待办事项
        user_account.todos.remove(index as usize);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init, // 创建新账户
        payer = author, // 账户创建费用由 author 支付
        space = 8 + 32 + 4 + (4 + 50) * 10, // 账户大小计算
        // 8：账户标识符（discriminator）
        // 32：author 字段大小（一个 Pubkey）
        // 4：todos 向量的长度字段
        // (4+50)*10：预留10个待办事项空间（每个待办事项：4字节长度<u32> + 50字节内容<假设每个待办事项最多50字符>）
        seeds = [b"user", author.key().as_ref()], // 使用 PDA（程序派生地址），确保每个用户只有一个账户
        // 为每个用户生成唯一地址
        // 不需要存储账户地址，可随时推导
        // 确保每个用户只有一个待办事项账户
        // 防止账户冲突
        bump // 自动计算bump值
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub author: Signer<'info>, // 交易签名者，标记为 mut 因为需要支付租金
    pub system_program: Program<'info, System>, // 必需的 Solana 系统程序引用
}

#[derive(Accounts)]
pub struct ManageTodo<'info> {
    #[account(
        mut, // 账户将被修改（状态变更）
        has_one = author, // 安全约束，确保 user_account 的 author 字段与当前 author 匹配
        seeds = [b"user", author.key().as_ref()], // 使用相同的种子公式（b"user" + author 公钥）查找账户
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub author: Signer<'info>,
    // 无 system_program，因为不创建新账户
}
