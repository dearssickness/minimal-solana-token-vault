use anchor_lang::prelude::*;

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
pub struct ExtendEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub extend_period: u64,
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