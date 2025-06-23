// mjvFkAaysJHDbEAvcJVd6Q2ZKb9kTeGPx6rAqVVQcUR 11 SOL
// 4UfxCcZRjKPTKAyXShuSPP3E41XPaNAz6hoWmijUxr4b 0 SOL

use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{self, read_keypair_file};
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;

fn main() {
    // 创建 solana 链接
    let rpc_url = "http://127.0.0.1:8899";
    let client = RpcClient::new(rpc_url);

    // 发送方 sender
    let sender = read_keypair_file("/Users/wangyueying/.config/solana/ad1.json").expect("failed");
    // 接收方
    let recipient_pubkey =
        Pubkey::from_str("4UfxCcZRjKPTKAyXShuSPP3E41XPaNAz6hoWmijUxr4b").unwrap();
    let amount = 1_000_000_000; // 1 SOL

    // 创建转账指令
    let transfer_instruction =
        system_instruction::transfer(&sender.pubkey(), &recipient_pubkey, amount);

    // 获取最后一个区块
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    // 创建交易
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    let result = client.send_and_confirm_transaction(&transaction);
    match result {
        Ok(signature) => println!("转账成功，交易签名：{}", signature),
        Err(err) => eprintln!("转账失败：{}", err),
    };

    // // 指定要查询的余额账户公钥
    // // 接受空投账户
    // let account_pubkey = Pubkey::from_str("mjvFkAaysJHDbEAvcJVd6Q2ZKb9kTeGPx6rAqVVQcUR").unwrap();
    // let amount = 1_000_000_000; // 1 SOL
    // match client.request_airdrop(&account_pubkey, amount) {
    //     Ok(signature) => println!("空投成功，交易签名：{}", signature),
    //     Err(err) => eprintln!("空投失败：{}", err),
    // }

    // // 获取账户余额
    // match client.get_balance(&account_pubkey) {
    //     Ok(balance) => println!("账户余额：{} lamports", balance),
    //     // 账户余额：10_000_000_000 lamports = 10 SOL
    //     Err(err) => eprintln!("获取账户余额失败：{}", err),
    // }
}
