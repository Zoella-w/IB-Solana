use anchor_lang::prelude::*;

// 声明 program id
declare_id!("J8U8eAx2nkd4zJdkEhpnxZsPQn8TM3GTojgDEeLFZcfd");

// 声明 program 模块
#[program]
pub mod a {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

// 声明 数据账户
#[derive(Accounts)]
pub struct Initialize {}
