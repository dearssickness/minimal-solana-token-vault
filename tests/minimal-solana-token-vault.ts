import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import idl from "../target/idl/minimal_solana_token_vault.json";
import { PublicKey, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("minimal_solana_token_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const programId = new PublicKey("67C39nnxzG6iu1BHBimYQabPyXrgMZ3Eqk5VzurcnP8Z"); // Replace with `anchor keys list` output
  //const program = new Program(idl as MinimumSolanaTokenVault,  programId, provider);
  const program = new Program(idl as Idl, provider);

  const user = Keypair.generate();
  let mint: PublicKey;
  let userTokenAccount: PublicKey;
  let vault: PublicKey;

  before(async () => {
    await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise((resolve) => setTimeout(resolve, 30000));

    mint = await createMint(provider.connection, user, user.publicKey, null, 9);
    userTokenAccount = await createAccount(provider.connection, user, mint, user.publicKey);
    //vault = await createAccount(provider.connection, user, mint, user.publicKey);
    vault = await createAccount(provider.connection, user, mint, programId);
    await mintTo(provider.connection, user, mint, userTokenAccount, user, 1000);
  });

  it("Deposits tokens to the vault", async () => {
    const depositAmount = 500;
    const userTokenAccountInfoBefore = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoBefore = await getAccount(provider.connection, vault);

    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        user: user.publicKey,
        userTokenAccount,
        vault,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const userTokenAccountInfoAfter = await getAccount(provider.connection, userTokenAccount);
    const vaultAccountInfoAfter = await getAccount(provider.connection, vault);

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
  });
});