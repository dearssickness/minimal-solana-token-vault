use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount};
use crate::{constants::*, errors::ErrorCode, events::ExtendEvent, state::UserVault};

#[derive(Accounts)]
pub struct Extend<'info> {
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, UserVault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"token_vault", user.key().as_ref()],
        bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,
}

pub fn handler(ctx: Context<Extend>, extend_period: u64) -> Result<()> {
    // Security checks
    require!(ctx.accounts.user.is_signer, ErrorCode::MissingSignature);
    require!(extend_period > 0 && extend_period <= MAX_EXTEND_PERIOD, ErrorCode::InvalidExtendPeriod);

    let user_vault = &mut ctx.accounts.user_vault;
    let new_unlock_timestamp = user_vault.unlock_timestamp
        .checked_add(extend_period as i64)
        .ok_or_else(|| error!(ErrorCode::ArithmeticError))?;
    user_vault.unlock_timestamp = new_unlock_timestamp;

    let clock = Clock::get()?;

    emit!(ExtendEvent {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.token_vault.key(),
        extend_period,
        unlock_timestamp: user_vault.unlock_timestamp,
        timestamp: clock.unix_timestamp,
    });
    Ok(())
}