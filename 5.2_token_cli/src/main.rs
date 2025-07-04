// use borsh::{BorshDeserialize, BorshSerialize};
// use solana_client::rpc_client::RpcClient;
// use solana_sdk::{
//     instruction::{AccountMeta, Instruction},
//     program_pack::Pack,
//     pubkey::Pubkey,
//     signature::{Keypair, Signer, read_keypair_file},
//     system_instruction::create_account,
//     transaction::Transaction,
// };
// use spl_token::{ID as TOKEN_PROGRAM_ID, instruction::initialize_mint2, state::Mint};
// use std::str::FromStr;

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub enum TokenInstruction {
//     CreateToken { decimals: u8 },
//     Mint { amount: u64 },
// }

// fn main() {}

// #[test]
// fn test_fn() {
//     println!("🏁 测试开始");

//     // 创建 solana 链接
//     let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
//     println!("🌐 RPC 客户端已创建");

//     let payer = read_keypair_file("/Users/wangyueying/.config/solana/ad2.json")
//         .expect("❌ 无法读取支付账户密钥文件");
//     println!("💰 支付账户: {}", payer.pubkey());

//     let program_id =
//         Pubkey::from_str("6GFGXwAFvNdqxibZEWdTGwPoKGiTCUUgG76ZSwj3NBL5").expect("❌ 无效的程序 ID");
//     println!("🖥️ 程序 ID: {}", program_id);

//     let mint_account = Keypair::new();
//     println!("🪙 Mint账户: {}", mint_account.pubkey());

//     // 调用 create_token 函数
//     match create_token(
//         &rpc_client,
//         &program_id,
//         &payer,
//         &mint_account,
//         &payer.pubkey(),
//         6,
//     ) {
//         Ok(_) => println!("✅ 测试成功"),
//         Err(e) => {
//             println!("❌ 测试失败: {:?}", e);
//             panic!("测试失败");
//         }
//     }
// }

// fn create_token(
//     rpc_client: &RpcClient,
//     program_id: &Pubkey,
//     payer: &Keypair,
//     mint_account: &Keypair,
//     mint_authority: &Pubkey,
//     decimals: u8,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     println!("\n🔛 进入 create_token 函数");

//     // 1. 确保支付账户有足够余额
//     ensure_sufficient_balance(rpc_client, payer)?;

//     println!("\n📊 计算账户租金...");
//     let mint_account_len = Mint::LEN;
//     let mint_account_rent = rpc_client.get_minimum_balance_for_rent_exemption(mint_account_len)?;
//     println!("  - Mint账户大小: {} 字节", mint_account_len);
//     println!("  - 所需租金: {} lamports", mint_account_rent);

//     println!("\n⚙️ 创建创建账户指令...");
//     let create_mint_account_ix = create_account(
//         &payer.pubkey(),
//         &mint_account.pubkey(),
//         mint_account_rent,
//         mint_account_len as u64,
//         &TOKEN_PROGRAM_ID,
//     );
//     println!("✅ 创建账户指令成功");

//     println!("\n⚙️ 创建初始化Mint指令...");
//     let initialize_mint_ix = initialize_mint2(
//         &TOKEN_PROGRAM_ID,
//         &mint_account.pubkey(),
//         mint_authority,
//         Some(mint_authority),
//         decimals,
//     )
//     .map_err(|e| {
//         println!("❌ initialize_mint2 失败: {:?}", e);
//         e
//     })?;
//     println!("✅ 初始化Mint指令成功");

//     println!("\n⏳ 检查区块哈希...");
//     let latest_blockhash = rpc_client.get_latest_blockhash().map_err(|e| {
//         println!("❌ 获取最新区块哈希失败: {:?}", e);
//         e
//     })?;
//     println!("✅ 最新区块哈希: {}", latest_blockhash);

//     println!("\n⚡ 创建交易...");
//     let mut transaction = Transaction::new_with_payer(
//         &[create_mint_account_ix, initialize_mint_ix],
//         Some(&payer.pubkey()),
//     );

//     println!("🔏 交易签名...");
//     transaction.sign(&[payer, mint_account], latest_blockhash);
//     println!("✅ 交易签名成功");

//     println!("\n📤 发送交易...");
//     let signature = rpc_client
//         .send_and_confirm_transaction_with_spinner(&transaction)
//         .map_err(|e| {
//             println!("❌ 发送交易失败: {:?}", e);
//             e
//         })?;
//     println!("✅ 交易成功! 签名: {}", signature);

//     println!("\n🔍 验证账户创建...");
//     match rpc_client.get_account(&mint_account.pubkey()) {
//         Ok(account) => println!("📊 Mint账户数据大小: {} 字节", account.data.len()),
//         Err(e) => println!("⚠️ Mint账户不存在: {:?}", e),
//     }

//     println!("\n🪙 代币创建成功!");
//     Ok(())
// }

// // 确保支付账户有足够余额
// fn ensure_sufficient_balance(
//     rpc_client: &RpcClient,
//     payer: &Keypair,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     const MIN_BALANCE: u64 = 100_000_000; // 0.1 SOL

//     let balance = rpc_client.get_balance(&payer.pubkey())?;
//     println!("💳 支付账户余额: {} lamports", balance);

//     if balance < MIN_BALANCE {
//         println!("⚠️ 余额不足，请求空投...");
//         let airdrop_amount = MIN_BALANCE - balance;

//         // 请求空投
//         let signature = rpc_client.request_airdrop(&payer.pubkey(), airdrop_amount)?;
//         println!("💸 空投已发送! 签名: {}", signature);

//         // 等待空投确认
//         println!("⏳ 等待空投确认...");
//         loop {
//             match rpc_client.confirm_transaction(&signature) {
//                 Ok(confirmed) if confirmed => {
//                     println!("✅ 空投已确认");
//                     break;
//                 }
//                 Ok(_) => {
//                     println!("⏳ 空投还未确认...");
//                 }
//                 Err(e) => println!("⚠️ 确认空投时出错: {:?}", e),
//             }
//             std::thread::sleep(std::time::Duration::from_secs(1));
//         }

//         // 验证新余额
//         let new_balance = rpc_client.get_balance(&payer.pubkey())?;
//         println!("💰 新余额: {} lamports", new_balance);
//     }

//     Ok(())
// }

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk::signature::read_keypair_file;
use solana_sdk::{
    signature::{Keypair, Signer},
    sysvar,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::{fmt::Debug, str::FromStr};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TokenInstruction {
    CreateToken { decimals: u8 },
    Mint { amount: u64 },
}

fn main() {}

#[test]
fn test_fn() {
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let payer = read_keypair_file("/Users/milo/.config/solana/ad2.json").expect("failed");

    let program_id = Pubkey::from_str("J1BeBUsTPQdbfxRTTSQEXjf1MAieGwKqAWvojbkQQKgg").unwrap(); // 你的程序ID

    let mint_account = Keypair::new();

    println!("{:?}", mint_account.to_base58_string());
    println!("{:?}", mint_account.pubkey().to_string());

    // 1. 创建SPL Token
    create_token(
        &rpc_client,
        &program_id,
        &payer,
        &mint_account,
        &payer.pubkey(),
        6,
    );

    // 2. mint
    mint(&rpc_client, &program_id, &mint_account, &payer, 1000000000);
}

fn create_token(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    mint_account: &Keypair,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    // 将指令序列化为字节数组
    let instruction_data = borsh::to_vec(&TokenInstruction::CreateToken { decimals }).unwrap();

    // 构建账户元数据
    let accounts = vec![
        AccountMeta::new(mint_account.pubkey(), true),
        AccountMeta::new_readonly(*mint_authority, false),
        AccountMeta::new_readonly(payer.pubkey(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    // 构建指令
    let token_instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // Step 4: Send the transaction
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[token_instruction],
        Some(&payer.pubkey()),
        &[payer, mint_account], // Sign with both payer and mint account keypairs
        latest_blockhash,
    );

    let r = rpc_client.send_and_confirm_transaction(&tx);
    println!("{:?}", r.unwrap());

    println!("Token created successfully.");
    Ok(())
}

fn mint(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    mint_account: &Keypair,
    payer: &Keypair,
    amount: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建ATA账号
    let ata = get_associated_token_address(&payer.pubkey(), &mint_account.pubkey());

    let instruction_data: Vec<u8> = borsh::to_vec(&TokenInstruction::Mint { amount }).unwrap();
    println!("{}", ata.to_string());
    let accounts = vec![
        AccountMeta::new(mint_account.pubkey(), true), // Mint 需要是可写的
        // AccountMeta::new_readonly(payer.pubkey(), true),  // Payer 需要是签名者
        AccountMeta::new(ata, false), // ATA 可写，但不需要 signer
        AccountMeta::new_readonly(sysvar::rent::id(), false), // Rent Sysvar
        AccountMeta::new(payer.pubkey(), true), // Payer 是可写的并且需要签名
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // 系统程序
        AccountMeta::new_readonly(spl_token::id(), false), // SPL Token 程序
        AccountMeta::new_readonly(spl_associated_token_account::id(), false), // SPL Token 程序
    ];

    // 构建指令
    let token_instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // 发送交易
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[token_instruction],
        Some(&payer.pubkey()),
        &[payer, mint_account], // 使用 Payer 和 Mint 的签名
        latest_blockhash,
    );

    let r = rpc_client.send_and_confirm_transaction(&tx);
    println!("{:?}", r.unwrap());

    println!("Token created successfully.");
    Ok(())
}
