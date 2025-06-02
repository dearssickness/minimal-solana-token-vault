use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{constants::*, errors::ErrorCode, events::DepositEvent, state::UserVault};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, UserVault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"token_vault", user.key().as_ref()],
        bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"vault-authority"],
        bump,
    )]
/// CHECK: This PDA is used only as a signing authority (token::authority), no data is read or written.
    pub vault_authority: AccountInfo<'info>,    
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Deposit>, lock_period: u64, amount: u64) -> Result<()> {
    // Security checks
    require!(ctx.accounts.user.is_signer, ErrorCode::MissingSignature);
    require!(lock_period > 0 && lock_period <= MAX_LOCK_PERIOD, ErrorCode::InvalidLockPeriod);

    let clock = Clock::get()?;
    let unlock_timestamp = clock.unix_timestamp + lock_period as i64;

    let user_vault = &mut ctx.accounts.user_vault;
    user_vault.lock_period = lock_period;
    user_vault.unlock_timestamp = unlock_timestamp;

    // Transfer SPL tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(DepositEvent {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.token_vault.key(),
        amount,
        lock_period,
        unlock_timestamp,
        timestamp: clock.unix_timestamp,
    });
    Ok(())
}