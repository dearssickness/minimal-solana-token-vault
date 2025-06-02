use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{events::FeeVaultInitialized};

#[derive(Accounts)]
pub struct InitializeFeeVault<'info> {
    #[account(
        init,
        payer = initializer,
        seeds = [b"fee_vault"],
        bump,
        token::mint = token_mint,
        token::authority = vault_authority,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
    /// CHECK: This PDA is used only as a signing authority, no data is read or written.
    #[account(
        seeds = [b"vault-authority"],
        bump,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeFeeVault>) -> Result<()> {
    emit!(FeeVaultInitialized {
        initializer: ctx.accounts.initializer.key(),
        fee_vault: ctx.accounts.fee_vault.key(),
        token_mint: ctx.accounts.token_mint.key(),
    });
    Ok(())
}