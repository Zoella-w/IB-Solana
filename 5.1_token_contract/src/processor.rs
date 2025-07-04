use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,                              // 用于在程序执行过程中输出日志信息
    program::{invoke, invoke_signed}, // 用于执行跨程序调用(CPI)
    pubkey::Pubkey,                   // Solana 的公钥类型
    system_instruction,               // 系统指令（如创建账户）
    sysvar::{Sysvar, rent::Rent},     // 系统变量（如租金计算）
};
use spl_token::{
    instruction::{initialize_mint, mint_to}, // SPL Token 的初始化铸币和铸造指令
    state::Mint,                             // SPL Token 的铸币账户状态
};

use std::str::FromStr;

use borsh::BorshDeserialize; // Borsh 反序列化库

use crate::instruction::TokenInstruction; // 自定义指令枚举
use solana_program::program_pack::Pack; // 用于获取账户数据大小的 trait

/// 代币处理器结构体
pub struct Processor;

impl Processor {
    /// 处理程序入口点
    ///
    /// # 参数
    /// - `_program_id`: 当前程序的ID
    /// - `accounts`: 传入的账户列表
    /// - `instruction_data`: 指令数据
    ///
    /// # 返回
    /// 程序执行结果
    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // 反序列化指令数据
        let instruction = TokenInstruction::try_from_slice(instruction_data)?;

        // 根据指令类型分发处理
        match instruction {
            TokenInstruction::CreateToken { decimals } => Self::create_token(accounts, decimals),
            TokenInstruction::Mint { amount } => Self::mint(accounts, amount),
        }
    }

    /// 创建代币（铸币账户）
    ///
    /// # 参数
    /// - `accounts`: 传入的账户列表
    /// - `decimals`: 代币的小数位数
    ///
    /// # 账户顺序要求
    /// 1. 铸币账户 (可写)
    /// 2. 铸币权限账户 (签名)
    /// 3. 支付账户 (签名)
    /// 4. 租金系统变量账户 (只读)
    /// 5. 系统程序账户 (只读)
    /// 6. 代币程序账户 (只读)
    fn create_token(accounts: &[AccountInfo], decimals: u8) -> ProgramResult {
        // 创建账户迭代器
        let accounts_iter = &mut accounts.iter();

        // 按顺序解析账户
        let mint_account = next_account_info(accounts_iter)?; // 铸币账户
        let mint_authority = next_account_info(accounts_iter)?; // 铸币权限账户
        let payer = next_account_info(accounts_iter)?; // 支付账户
        let rent_sysvar = next_account_info(accounts_iter)?; // 租金系统变量
        let system_program = next_account_info(accounts_iter)?; // 系统程序
        let token_program = next_account_info(accounts_iter)?; // 代币程序

        // 日志输出
        msg!("Creating mint account...");
        msg!("Mint: {}", mint_account.key);

        // 创建铸币账户 - 使用系统指令
        // 关键API: system_instruction::create_account
        // 功能: 创建一个新账户
        // 参数:
        //   payer.key - 支付账户
        //   mint_account.key - 新账户地址
        //   Rent::get()?.minimum_balance(Mint::LEN) - 所需租金
        //   Mint::LEN as u64 - 账户大小
        //   token_program.key - 账户所有者（代币程序）
        invoke(
            &system_instruction::create_account(
                payer.key,
                mint_account.key,
                (Rent::get()?).minimum_balance(Mint::LEN), // 计算租金
                Mint::LEN as u64,                          // 账户大小
                token_program.key,                         // 账户所有者
            ),
            &[
                mint_account.clone(),
                payer.clone(),
                system_program.clone(),
                token_program.clone(),
            ],
        )?;

        // 获取租金信息（虽然未使用，但展示了如何从账户获取租金）
        let _rent = Rent::from_account_info(rent_sysvar)?;

        // 初始化铸币账户 - 使用SPL Token指令
        // 关键API: initialize_mint
        // 功能: 初始化铸币账户
        // 参数:
        //   &spl_token::id() - 代币程序ID
        //   &mint_account.key - 铸币账户地址
        //   &mint_authority.key - 铸币权限
        //   None - 冻结权限（无）
        //   decimals - 小数位数
        let ix = initialize_mint(
            &spl_token::id(),
            &mint_account.key,
            &mint_authority.key,
            None,     // 冻结权限
            decimals, // 小数位数
        )?;

        // 日志输出
        msg!("Initializing mint account...");
        msg!("Mint: {}", mint_account.key);

        // 执行初始化指令 - 使用跨程序调用
        // 关键API: invoke_signed
        // 功能: 执行带有签名的跨程序调用
        // 参数:
        //   &ix - 要执行的指令
        //   账户列表 - 指令所需的账户
        //   &[] - 签名者种子（此处为空）
        invoke_signed(
            &ix,
            &[
                mint_account.clone(),
                rent_sysvar.clone(), // 注意：SPL Token 初始化需要租金系统变量
                token_program.clone(),
                mint_authority.clone(),
            ],
            &[], // 不需要额外签名
        )?;

        // 成功日志
        msg!("SPL Token Mint created successfully");

        Ok(())
    }

    /// 铸造代币到关联令牌账户
    ///
    /// # 参数
    /// - `accounts`: 传入的账户列表
    /// - `amount`: 要铸造的代币数量
    ///
    /// # 账户顺序要求
    /// 1. 铸币账户 (可写)
    /// 2. 关联令牌账户 (可写)
    /// 3. 租金系统变量账户 (只读)
    /// 4. 支付账户 (签名)
    /// 5. 系统程序账户 (只读)
    /// 6. 代币程序账户 (只读)
    /// 7. 关联令牌账户程序 (只读)
    pub fn mint(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        // 创建账户迭代器
        let accounts_iter = &mut accounts.iter();

        // 按顺序解析账户
        let mint_account = next_account_info(accounts_iter)?; // 铸币账户
        let associated_token_account = next_account_info(accounts_iter)?; // 关联令牌账户
        let rent_sysvar = next_account_info(accounts_iter)?; // 租金系统变量
        let payer = next_account_info(accounts_iter)?; // 支付账户
        let system_program = next_account_info(accounts_iter)?; // 系统程序
        let token_program = next_account_info(accounts_iter)?; // 代币程序
        let associated_token_program = next_account_info(accounts_iter)?; // 关联令牌账户程序

        // 调试输出
        msg!("{:?}", associated_token_account);

        // 检查关联令牌账户是否存在（通过lamports判断）
        if associated_token_account.lamports() == 0 {
            // 如果不存在，创建关联令牌账户
            msg!("Creating associated token account...");

            // 关键API: spl_associated_token_account::instruction::create_associated_token_account
            // 功能: 创建关联令牌账户指令
            // 参数:
            //   payer.key - 支付账户
            //   payer.key - 代币所有者（此处与支付账户相同）
            //   mint_account.key - 铸币账户
            //   token_program.key - 代币程序
            invoke(
                &spl_associated_token_account::instruction::create_associated_token_account(
                    payer.key,
                    payer.key, // 代币所有者
                    mint_account.key,
                    token_program.key,
                ),
                &[
                    payer.clone(),
                    associated_token_account.clone(),
                    mint_account.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    rent_sysvar.clone(),
                    associated_token_program.clone(),
                ],
            )?;
        } else {
            msg!("Associated token account exists.");
        }

        // 输出关联令牌账户地址
        msg!("Associated Token Address: {}", associated_token_account.key);

        // 铸造代币到关联令牌账户
        msg!("Minting {} tokens to associated token account...", amount);

        // 关键API: mint_to
        // 功能: 创建铸造代币指令
        // 参数:
        //   token_program.key - 代币程序
        //   mint_account.key - 铸币账户
        //   associated_token_account.key - 目标账户
        //   payer.key - 铸造权限
        //   &[payer.key] - 签名者
        //   amount - 铸造数量
        invoke(
            &mint_to(
                token_program.key,
                mint_account.key,
                associated_token_account.key,
                payer.key,    // 铸造权限
                &[payer.key], // 签名者
                amount,
            )?,
            &[
                mint_account.clone(),
                payer.clone(),
                associated_token_account.clone(),
                token_program.clone(),
            ],
        )?;

        // 成功日志
        msg!("Tokens minted to wallet successfully.");

        Ok(())
    }
}
