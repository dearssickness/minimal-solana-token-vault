use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
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
    #[msg("Invalid lock period")]
    InvalidLockPeriod,
    #[msg("Invalid extend period")]
    InvalidExtendPeriod,
}