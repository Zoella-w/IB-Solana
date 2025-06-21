use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, Solana!");
    msg!("Our program's Program ID: {}", &program_id);
    msg!("program_id: {:?}", program_id);
    msg!("_accounts: {:?}", _accounts);
    msg!("instruction_data: {:?}", _instruction_data);
    Ok(())
}
