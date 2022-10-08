import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID, createMint, getAccount, mintTo, getOrCreateAssociatedTokenAccount, transfer } from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js'
const serumCmn = require("@project-serum/common");
import { Bond } from "../target/types/bond";
const BN = anchor.BN;
const encode = anchor.utils.bytes.utf8.encode;
import { assert } from 'chai'


describe("bond", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const program = anchor.workspace.Bond as Program<Bond>;

  // let mint = new PublicKey("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T");
  it("Is initialized!", async () => {
    const [usdcVault, usdcBump] = await PublicKey.findProgramAddress(
      [
        encode("vault_token"),
        // USDC.toBuffer(),
      ],
      program.programId
    );
    // Generate a new wallet keypair and airdrop SOL
    const fromWallet = Keypair.generate();
    const connection = provider.connection;
    const fromAirdropSignature = await connection.requestAirdrop(fromWallet.publicKey, LAMPORTS_PER_SOL);

    // Wait for airdrop confirmation
    await connection.confirmTransaction(fromAirdropSignature);

    // Generate a new wallet to receive newly minted token
    const toWallet = Keypair.generate();

    // Create new token mint
    const mint = await createMint(connection, fromWallet, fromWallet.publicKey, null, 9);

    // Get the token account of the fromWallet address, and if it does not exist, create it
    const fromTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      fromWallet,
      mint,
      fromWallet.publicKey
    );

    // Get the token account of the toWallet address, and if it does not exist, create it
    const toTokenAccount = await getOrCreateAssociatedTokenAccount(connection, fromWallet, mint, toWallet.publicKey);

    // Mint 1 new token to the "fromTokenAccount" account we just created
    let signature = await mintTo(
      connection,
      fromWallet,
      mint,
      fromTokenAccount.address,
      fromWallet.publicKey,
      1000000000
    );
    console.log('mint tx:', signature);
    // Transfer the new token to the "toTokenAccount" we just created
    signature = await transfer(
      connection,
      fromWallet,
      fromTokenAccount.address,
      toTokenAccount.address,
      fromWallet.publicKey,
      50
    );
    // Add your test here.
    const tx = await program.methods.initializeVault(usdcBump).accounts({
      tokenMint: mint,
      vaultAccount: usdcVault,
    }).signers([]).rpc();
    let vault_account = await getAccount(provider.connection, usdcVault)
    let [vault_pda, _] = await PublicKey.findProgramAddress([encode("scale_vault")], program.programId)
    assert.isTrue(vault_account.owner.equals(vault_pda));
    assert.isTrue(vault_account.mint.equals(mint));
    assert.isTrue(vault_account.address.equals(usdcVault));
    console.log("Your transaction signature", tx, vault_account, vault_pda);
  });
  it("market is initialized!", async () => {

  });
});

