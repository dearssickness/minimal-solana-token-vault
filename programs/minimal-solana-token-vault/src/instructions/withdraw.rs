use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{errors::ErrorCode, events::WithdrawEvent, state::UserVault};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
    #[account(
        mut,
        seeds = [b"fee_vault"],
        bump,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Security checks
    require!(ctx.accounts.user.is_signer, ErrorCode::MissingSignature);

    let clock = Clock::get()?;
    let user_vault = &ctx.accounts.user_vault;

    let vault_balance = ctx.accounts.token_vault.amount;
    require!(vault_balance >= amount, ErrorCode::InsufficientVaultBalance);

    let vault_authority_seeds = &[
        b"vault-authority".as_ref(),
        &[ctx.bumps.vault_authority],
    ];

    // Determine fee based on lock status
    let is_locked = clock.unix_timestamp < user_vault.unlock_timestamp;
    let fee_percentage = if is_locked { 5 } else { 1 }; // 5% fee if locked, 1% if unlocked

    let fee = amount
        .checked_mul(fee_percentage)
        .ok_or_else(|| error!(ErrorCode::ArithmeticError))?
        .checked_div(100)
        .ok_or_else(|| error!(ErrorCode::ArithmeticError))?;

    let amount_after_fee = amount
        .checked_sub(fee)
        .ok_or_else(|| error!(ErrorCode::InsufficientAmount))?;

    // Transfer fee to fee_vault
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.fee_vault.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&vault_authority_seeds[..]],
        ),
        fee,
    )?;

    // Transfer remaining amount to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&vault_authority_seeds[..]],
        ),
        amount_after_fee,
    )?;

    emit!(WithdrawEvent {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.token_vault.key(),
        amount,
        fee,
        amount_after_fee,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}