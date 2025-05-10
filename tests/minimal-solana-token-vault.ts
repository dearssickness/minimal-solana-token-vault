import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import idl from "../target/idl/minimal_solana_token_vault.json";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

describe("minimal_solana_token_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programId = new PublicKey("B1Lg1HExK1jPRrQfb9vPGxpBKgVJCbjANNiQqYGFK1kq");
  const program = new Program(idl as Idl, provider);
  const user = Keypair.generate();

  let mint: PublicKey;
  let userTokenAccount: PublicKey;
  let token_vault: PublicKey;
  let user_vault: PublicKey;
  let vault_authority: PublicKey;
  let fee_vault: PublicKey;

  before(async () => {
    const airdropSignature = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    })

    mint = await createMint(provider.connection, user, user.publicKey, null, 9);

    userTokenAccount = await createAccount(provider.connection, user, mint, user.publicKey);


    const [vaultPda] = findProgramAddressSync([Buffer.from("user_vault"), user.publicKey.toBuffer()],programId)
    user_vault = vaultPda;
    
    const [tokenPda] = findProgramAddressSync([Buffer.from("token_vault"), user.publicKey.toBuffer()],programId)
    token_vault = tokenPda;

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
        token_vault,
        vault_authority,
        tokenMint: mint,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    await mintTo(provider.connection, user, mint, userTokenAccount, user, 1000);
  });

  it("Deposits tokens to the vault", async () => {
  
      const listenerId = program.addEventListener("depositEvent", event => {
        // Do something with the event data
        console.log("Event Data:", event);
      });

    const lockPeriod = 5;
    const depositAmount = 500;

    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoBefore = await getAccount(provider.connection, token_vault);

    await program.methods
      .deposit(new anchor.BN(lockPeriod), new anchor.BN(depositAmount))
      .accounts({
        user: user.publicKey,
        userTokenAccount,
        token_vault,
        user_vault,
        vault_authority,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoAfter = await getAccount(provider.connection, token_vault);

    console.log("userTokenAccountInfoBefore: ", userTokenAccountInfoBefore.amount);
    console.log("userTokenAccountInfoAfter: ", userTokenAccountInfoAfter.amount);
    console.log("vaultAccountInfoBefore: ", vaultAccountInfoBefore.amount);
    console.log("vaultAccountInfoAfter: ", vaultAccountInfoAfter.amount);
 
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

    await program.removeEventListener(listenerId);

  });

  it("Withdraw tokens from the vault in locked period", async () => {
    const withdrawAmount = 100;

    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);

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
      .rpc()

    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const feeVaultInfoAfter = await getAccount(provider.connection, fee_vault);

    console.log("feeVaultInfoAfter 5% fee: ", feeVaultInfoAfter.amount);
    
    assert.equal(
      Number(userTokenAccountInfoBefore.amount) + withdrawAmount - ( withdrawAmount * 5 / 100),
      Number(userTokenAccountInfoAfter.amount),
      "User token account should increase"
    );
  });

  it("Withdraw tokens from the vault after locked period", async () => {
    const withdrawAmount = 100;

    await new Promise(f => setTimeout(f, 6000));    

    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);

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
      .rpc()

    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const feeVaultInfoAfter = await getAccount(provider.connection, fee_vault);

    console.log("feeVaultInfoAfter 1% fee: ", feeVaultInfoAfter.amount);

    assert.equal(
      Number(userTokenAccountInfoBefore.amount) + withdrawAmount - (withdrawAmount / 100),
      Number(userTokenAccountInfoAfter.amount),
      "User token account should increase"
    );

  });

  it("Extend locked period", async () => {
    const extend_duration = 100; // Extend by 100 seconds
    const userVaultBefore = await program.account.userVault.fetch(user_vault); // Fetch user_vault account
  
    await program.methods
      .extend(new anchor.BN(extend_duration))
      .accounts({
        user: user.publicKey,
        user_vault,
        token_vault,
      })
      .signers([user])
      .rpc();
  
    const userVaultAfter = await program.account.userVault.fetch(user_vault);
    
    console.log("userVaultBefore: ", userVaultBefore.unlockTimestamp.toNumber())
    console.log("userVaultAfter: ", userVaultAfter.unlockTimestamp.toNumber())
  
    assert.equal(
      userVaultBefore.unlockTimestamp.toNumber() + extend_duration,
      userVaultAfter.unlockTimestamp.toNumber(),
      "Unlock timestamp should extend by the specified duration"
    );
  });

});