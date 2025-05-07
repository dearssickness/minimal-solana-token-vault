# Minimal Solana Token Vault

This is a lightweight SPL token vault built with [Anchor](https://www.anchor-lang.com/), designed for simple deposit and withdrawal flows with fee handling on the Solana blockchain.

## Features

- Initialize a fee vault (to collect withdrawal fees)
- Initialize user-specific token vaults
- Deposit SPL tokens into a vault
- Withdraw tokens with automatic 1% fee deduction

---

## How to Deploy

### 1. Install Prerequisites

Make sure you have these installed:

- **Rust**
- **Solana CLI**
- **Anchor**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Anchor
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.0 anchor-cli --locked

solana config set --url https://api.devnet.solana.com
solana-keygen new

```

There is a known [bug](https://github.com/solana-foundation/anchor/pull/3663#issuecomment-2810222442) in anchor 0.31.0 that can be temporarily fixed with
this [fix](https://github.com/solana-foundation/anchor/pull/3663#issuecomment-2810358025).
After applying the fix you can do:


```bash
npm i -D tsx
anchor build
anchor deploy

solana program show <PROGRAM_ID>
```

## Program Instructions
initialize_fee_vault
Creates a vault account to collect fees from all withdrawals.

initialize_vault
Creates a user-specific vault to store SPL tokens.

deposit(amount: u64)
Transfers tokens from the user’s token account to their vault.

withdraw(amount: u64)
Withdraws tokens from the vault to the user’s token account, taking 1% as a fee into the fee_vault.

## Testing
anchor test

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup