use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("67C39nnxzG6iu1BHBimYQabPyXrgMZ3Eqk5VzurcnP8Z"); // Replace with `anchor keys list` output

#[error_code]
pub enum ErrorCode{
    #[msg("Arithmetic error during fee calculation")]
    ArithmeticError,
    #[msg("Insufficient amount to cover fee")]
    InsufficientAmount,
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
            vault: ctx.accounts.user_vault.key(),
            token_mint: ctx.accounts.token_mint.key(),
        });
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Transfer SPL tokens from user to vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.user_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            vault: ctx.accounts.user_vault.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Define the seeds for the vault_authority PDA
        let vault_authority_seeds = &[
            b"vault-authority".as_ref(),
            &[ctx.bumps.vault_authority], // Use the bump from the context
        ];
        
        let fee = amount
            .checked_div(100)
            .ok_or_else(|| error!(ErrorCode::ArithmeticError))?;
        
        let amount_after_fee = amount
            .checked_sub(fee)
            .ok_or_else(|| error!(ErrorCode::InsufficientAmount))?;
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer{
                    from:ctx.accounts.user_vault.to_account_info(),
                    to:ctx.accounts.fee_vault.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[&vault_authority_seeds[..]], // Pass the seeds for PDA signing
            ),
            fee,
        )?;
 
        // Transfer SPL tokens from vault to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_vault.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[&vault_authority_seeds[..]], // Pass the seeds for PDA signing
            ),
            amount_after_fee,
        )?;
        emit!(WithdrawEvent {
            user: ctx.accounts.user.key(),
            vault: ctx.accounts.user_vault.key(),
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
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = vault_authority,
    )]
    pub user_vault: Account<'info, TokenAccount>,
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
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, TokenAccount>,
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
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump,
    )]
    pub user_vault: Account<'info, TokenAccount>,
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