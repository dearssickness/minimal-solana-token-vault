#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("B1Lg1HExK1jPRrQfb9vPGxpBKgVJCbjANNiQqYGFK1kq");

pub mod constants;
pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod minimal_solana_token_vault {
    use super::*;

    pub fn initialize_fee_vault(ctx: Context<InitializeFeeVault>) -> Result<()> {
        instructions::initialize_fee_vault::handler(ctx)
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, lock_period: u64, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, lock_period, amount)
    }

    pub fn extend(ctx: Context<Extend>, extend_period: u64) -> Result<()> {
        instructions::extend::handler(ctx, extend_period)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }
}