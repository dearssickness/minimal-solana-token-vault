use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{events::VaultInitialized, state::UserVault};

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + 8 + 8,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, UserVault>,
    #[account(
        init,
        payer = user,
        seeds = [b"token_vault", user.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = vault_authority,
    )]
    pub token_vault: Account<'info, TokenAccount>,
    /// CHECK: This PDA is used only as a signing authority (token::authority), no data is read or written.
    #[account(
        seeds = [b"vault-authority"],
        bump,
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVault>) -> Result<()> {
    emit!(VaultInitialized {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.token_vault.key(),
        token_mint: ctx.accounts.token_mint.key(),
    });
    Ok(())
}