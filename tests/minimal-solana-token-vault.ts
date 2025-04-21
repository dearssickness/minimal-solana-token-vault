import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import idl from "../target/idl/minimal_solana_token_vault.json";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert } from "chai";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

describe("minimal_solana_token_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programId = new PublicKey("67C39nnxzG6iu1BHBimYQabPyXrgMZ3Eqk5VzurcnP8Z");
  const program = new Program(idl as Idl, provider);
  const user = Keypair.generate();
  const user_2 = Keypair.generate();

  let mint: PublicKey;
  let mint_user_2: PublicKey;
  let userTokenAccount: PublicKey;
  let userTokenAccount_2: PublicKey;
  let user_vault: PublicKey;
  let user_vault_2: PublicKey;
  let vault_authority: PublicKey;
  let fee_vault: PublicKey;

  before(async () => {
    const airdropSignature = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const airdropSignature_user_2 = await provider.connection.requestAirdrop(user_2.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    })

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature_user_2,
    })

    mint = await createMint(provider.connection, user, user.publicKey, null, 9);
    mint_user_2 = await createMint(provider.connection, user_2, user_2.publicKey, null, 9);

    userTokenAccount = await createAccount(provider.connection, user, mint, user.publicKey);
    userTokenAccount_2 = await createAccount(provider.connection, user_2, mint_user_2, user_2.publicKey);

    const [vaultPda] = findProgramAddressSync([Buffer.from("user_vault"), user.publicKey.toBuffer()],programId)
    user_vault = vaultPda;
    
    const [vaultPda_user_2] = findProgramAddressSync([Buffer.from("user_vault"), user_2.publicKey.toBuffer()],programId)
    user_vault_2 = vaultPda_user_2;

    const [vaultAuthorityPda] = findProgramAddressSync([Buffer.from("vault_authority")],programId)
    vault_authority = vaultAuthorityPda;

    const [feeVaultPda] = findProgramAddressSync([Buffer.from("fee_vault")],programId)
    fee_vault = feeVaultPda;
    
    await program.methods
      .initializeFeeVault()
      .accounts({
        fee_vault,
        vault_authority,
        initializer: user.publicKey,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    await program.methods
      .initializeVault()
      .accounts({
        user_vault,
        vault_authority,
        tokenMint: mint,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    await program.methods
      .initializeVault()
      .accounts({
        user_vault_2,
        vault_authority,
        tokenMint: mint_user_2,
        user: user_2.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user_2])
      .rpc();

    await mintTo(provider.connection, user, mint, userTokenAccount, user, 1000);
    await mintTo(provider.connection, user_2, mint_user_2, userTokenAccount_2, user_2, 700);
  });

  it("Deposits tokens to the vault", async () => {
  
      const listenerId = program.addEventListener("depositEvent", event => {
        // Do something with the event data
        console.log("Event Data:", event);
      });

    const depositAmount = 500;
    const depositAmount_user_2 = 300;

    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoBefore = await getAccount(provider.connection, user_vault);

    const userTokenAccountInfoBefore_user_2 = await getAccount(provider.connection, userTokenAccount_2);
    const vaultAccountInfoBefore_user_2 = await getAccount(provider.connection, user_vault_2);

    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        user: user.publicKey,
        userTokenAccount,
        user_vault,
        vault_authority,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    await program.methods
      .deposit(new anchor.BN(depositAmount_user_2))
      .accounts({
        user: user_2.publicKey,
        userTokenAccount: userTokenAccount_2,
        user_vault_2,
        vault_authority,
        tokenMint: mint_user_2,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_2])
      .rpc();

    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoAfter = await getAccount(provider.connection, user_vault);

    const userTokenAccountInfoAfter_user_2 = await getAccount(provider.connection, userTokenAccount_2);
    const vaultAccountInfoAfter_user_2 = await getAccount(provider.connection, user_vault_2);

    console.log("userTokenAccountInfoBefore: ", userTokenAccountInfoBefore.amount);
    console.log("userTokenAccountInfoAfter: ", userTokenAccountInfoAfter.amount);
    console.log("vaultAccountInfoBefore: ", vaultAccountInfoBefore.amount);
    console.log("vaultAccountInfoAfter: ", vaultAccountInfoAfter.amount);
    
    console.log("userTokenAccountInfoBefore_user_2: ", userTokenAccountInfoBefore_user_2.amount);
    console.log("userTokenAccountInfoAfter_user_2: ", userTokenAccountInfoAfter_user_2.amount);
    console.log("vaultAccountInfoBefore_user_2: ", vaultAccountInfoBefore_user_2.amount);
    console.log("vaultAccountInfoAfter_user_2: ", vaultAccountInfoAfter_user_2.amount);
 
    assert.equal(
      Number(userTokenAccountInfoBefore.amount) - depositAmount,
      Number(userTokenAccountInfoAfter.amount),
      "User token account should decrease"
    );
    assert.equal(
      Number(vaultAccountInfoBefore.amount) + depositAmount,
      Number(vaultAccountInfoAfter.amount),
      "Vault token account should increase"
    );

    assert.equal(
      Number(userTokenAccountInfoBefore_user_2.amount) - depositAmount_user_2,
      Number(userTokenAccountInfoAfter_user_2.amount),
      "User 2 token account should decrease"
    );
    assert.equal(
      Number(vaultAccountInfoBefore_user_2.amount) + depositAmount_user_2,
      Number(vaultAccountInfoAfter_user_2.amount),
      "User 2 Vault's token account should increase"
    );

    await program.removeEventListener(listenerId);

  });

  it("Withdraw tokens from the vault", async () => {
    const withdrawAmount = 100;
    const fee = withdrawAmount / 100;
  
    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoBefore = await getAccount(provider.connection, user_vault);
    const feeVaultInfoBefore = await getAccount(provider.connection, fee_vault);
  
    await program.methods
      .withdraw(new anchor.BN(withdrawAmount))
      .accounts({
        user: user.publicKey,
        userTokenAccount,
        user_vault,
        vault_authority,
        fee_vault,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
  
    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoAfter = await getAccount(provider.connection, user_vault);
    const feeVaultInfoAfter = await getAccount(provider.connection, fee_vault);
  
    console.log("userTokenAccountInfoBefore: ", userTokenAccountInfoBefore.amount);
    console.log("userTokenAccountInfoAfter: ", userTokenAccountInfoAfter.amount);
    console.log("vaultAccountInfoBefore: ", vaultAccountInfoBefore.amount);
    console.log("vaultAccountInfoAfter: ", vaultAccountInfoAfter.amount);
    console.log("feeVaultInfoBefore: ", feeVaultInfoBefore.amount);
    console.log("feeVaultInfoAfter: ", feeVaultInfoAfter.amount);

    const userTokenAccountInfoBefore_user_2 = await getAccount(provider.connection, userTokenAccount_2);
    const vaultAccountInfoBefore_user_2 = await getAccount(provider.connection, user_vault_2);

    const userTokenAccountInfoAfter_user_2 = await getAccount(provider.connection, userTokenAccount_2);
    const vaultAccountInfoAfter_user_2 = await getAccount(provider.connection, user_vault_2);

    console.log("userTokenAccountInfoBefore_user_2: ", userTokenAccountInfoBefore_user_2.amount);
    console.log("userTokenAccountInfoAfter_user_2: ", userTokenAccountInfoAfter_user_2.amount);
    console.log("vaultAccountInfoBefore_user_2: ", vaultAccountInfoBefore_user_2.amount);
    console.log("vaultAccountInfoAfter_user_2: ", vaultAccountInfoAfter_user_2.amount);
  
    assert.equal(
      Number(userTokenAccountInfoBefore.amount) + withdrawAmount - fee,
      Number(userTokenAccountInfoAfter.amount),
      "User token account should increase"
    );
    assert.equal(
      Number(vaultAccountInfoBefore.amount) - withdrawAmount,
      Number(vaultAccountInfoAfter.amount),
      "Vault token account should decrease"
    );

    assert.equal(
      Number(userTokenAccountInfoBefore_user_2.amount),
      Number(userTokenAccountInfoAfter_user_2.amount),
      "User 2 token account should not change"
    );
    assert.equal(
      Number(vaultAccountInfoBefore_user_2.amount),
      Number(vaultAccountInfoAfter_user_2.amount),
      "User 2 Vault's token account should not change"
    );

  });

});