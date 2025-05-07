use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("B1Lg1HExK1jPRrQfb9vPGxpBKgVJCbjANNiQqYGFK1kq");

#[error_code]
pub enum ErrorCode{
    #[msg("Arithmetic error during fee calculation")]
    ArithmeticError,
    #[msg("Insufficient amount to cover fee")]
    InsufficientAmount,
    #[msg("User signature is missing")]
    MissingSignature,
    #[msg("Deposit is still locked")]
    DepositLocked,
    #[msg("Insufficient balance in vault")]
    InsufficientVaultBalance,
}

#[event]
pub struct VaultInitialized {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub token_mint: Pubkey,
}

#[event]
pub struct FeeVaultInitialized {
    pub initializer: Pubkey,
    pub fee_vault: Pubkey,
    pub token_mint: Pubkey,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub lock_period: u64,
    pub unlock_timestamp: i64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub amount_after_fee: u64,
    pub timestamp: i64,
}

#[program]
pub mod minimal_solana_token_vault {
//    use anchor_lang::solana_program::clock;

    use super::*;

    pub fn initialize_fee_vault(ctx: Context<InitializeFeeVault>) -> Result<()>{
        emit!(FeeVaultInitialized {
            initializer: ctx.accounts.initializer.key(),
            fee_vault: ctx.accounts.fee_vault.key(),
            token_mint: ctx.accounts.token_mint.key(),
        });
        Ok(())
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()>{
        emit!(VaultInitialized {
            user: ctx.accounts.user.key(),
            vault: ctx.accounts.token_vault.key(),
            token_mint: ctx.accounts.token_mint.key(),
        });
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, lock_period: u64, amount: u64) -> Result<()> {
        // Security checks
        require!(ctx.accounts.user.is_signer, ErrorCode::MissingSignature);
        
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
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
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
        let fee_percentage = if is_locked {5} else {1}; // 5% fee if locked, 1% if unlocked
        
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
                token::Transfer{
                    from:ctx.accounts.token_vault.to_account_info(),
                    to:ctx.accounts.fee_vault.to_account_info(),
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
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

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

#[account]
pub struct UserVault{
    pub user: Pubkey,
    pub token_mint: Pubkey,
    pub lock_period: u64,
    pub unlock_timestamp: i64,
}