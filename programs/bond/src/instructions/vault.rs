use crate::com;
use anchor_lang::prelude::*;
use anchor_spl::{
    mint,
    token::{self, spl_token::instruction::AuthorityType, Mint, SetAuthority, Token, TokenAccount},
};
// the vault spl token pda account
const VAULT_ACCOUNT_SEED: &[u8] = b"scale_vault";
pub fn initialize_vault(ctx: Context<InitializeVault>, bump: u8) -> Result<Pubkey> {
    let (pda_vault_account, _bump) =
        Pubkey::find_program_address(&[VAULT_ACCOUNT_SEED], ctx.program_id);
    token::set_authority(
        ctx.accounts.into(),
        AuthorityType::AccountOwner,
        Some(pda_vault_account),
    )?;
    msg!(
        "pda_vault_account: {:?} ,vault_account: {:?},bump:{:?}====>{:?}",
        pda_vault_account.key(),
        ctx.accounts.vault_account.key(),
        bump,
        ctx.accounts.vault_account.owner,
    );
    Ok(pda_vault_account.key())
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        seeds = [b"vault_token"],
        bump,
        payer=initializer,
        token::mint=token_mint,
        token::authority=initializer,
    )]
    pub vault_account: Account<'info, TokenAccount>,
    // #[account(address=mint::USDC)]
    // #[account(address=com::get_vault_mint())]
    pub token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> From<&mut InitializeVault<'info>>
    for CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>
{
    fn from(accounts: &mut InitializeVault<'info>) -> Self {
        let cpi_accounts = SetAuthority {
            account_or_mint: accounts.vault_account.to_account_info().clone(),
            current_authority: accounts.initializer.to_account_info().clone(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}