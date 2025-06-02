use anchor_lang::prelude::*;

#[account]
pub struct UserVault {
    pub user: Pubkey,
    pub token_mint: Pubkey,
    pub lock_period: u64,
    pub unlock_timestamp: i64,
}