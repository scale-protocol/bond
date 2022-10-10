import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getAccount, transfer } from "@solana/spl-token";
import { PublicKey, } from '@solana/web3.js'
const serumCmn = require("@project-serum/common");
import { Bond } from "../target/types/bond";
const BN = anchor.BN;
const encode = anchor.utils.bytes.utf8.encode;
import { assert, expect } from 'chai';
import initSplAccounts from "./utils";

const VAULT_TOKEN_ACCOUNT_SEED = encode("vault_token");

const VAULT_TOKEN_AUTHORITY_SEED = encode("scale_vault");

const USER_ACCOUNT_SEED = encode("scale_user_account");

const MARKET_ACCOUNT_SEED = encode("scale_vault_market");

const POSITION_ACCOUNT_SEED = encode("scale_position");

describe("bond", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Bond as Program<Bond>;
  var SPL;

  // let mint = new PublicKey("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T");
  it("test vault account init", async () => {
    const SPL_ACCOUNT = await initSplAccounts({ provider: provider })
    SPL = SPL_ACCOUNT
    const [usdcVault, usdcBump] = await PublicKey.findProgramAddress(
      [
        VAULT_TOKEN_ACCOUNT_SEED,
        // USDC.toBuffer(),
      ],
      program.programId
    );
    // Add your test here.
    const tx = await program.methods.initializeVault(usdcBump).accounts({
      tokenMint: SPL_ACCOUNT.mint,
      vaultAccount: usdcVault,
    }).signers([]).rpc();
    let [vault_pda, _] = await PublicKey.findProgramAddress([VAULT_TOKEN_AUTHORITY_SEED], program.programId)
    let vault_account = await getAccount(provider.connection, usdcVault)
    assert.isTrue(vault_account.owner.equals(vault_pda));
    assert.isTrue(vault_account.mint.equals(SPL_ACCOUNT.mint));
    assert.isTrue(vault_account.address.equals(usdcVault));
    console.log(
      "\nvault_account:", vault_account.address.toBase58(),
      "\nvault_account.mint:", vault_account.mint.toBase58(),
      "\nspl token mint:", SPL_ACCOUNT.mint.toBase58(),
      "\nvault_account_owner:", vault_account.owner.toBase58(),
      "\namount:", vault_account.amount,
      "\nvault_pda:", vault_pda.toBase58());
    const xx = await transfer(
      provider.connection,
      SPL_ACCOUNT.fromWallet,
      SPL_ACCOUNT.fromTokenAccount.address,
      new PublicKey("V5apPo62k57ap6h6c5uQnX2T6scJDaXap1dwhj7xomx"),
      SPL_ACCOUNT.fromWallet.publicKey,
      50000000000
    );
    let vault_account_1 = await getAccount(provider.connection, usdcVault)
    assert.strictEqual(vault_account_1.amount, BigInt(50000000000));
    console.log("\ntest vault token account:", xx, "\namount:", vault_account_1.amount)
  });



  it("test market account init", async () => {
    let [market_account, bump] = await PublicKey.findProgramAddress([MARKET_ACCOUNT_SEED, encode("BTC")], program.programId)
    const tx = await program.methods.initializeMarket(
      "BTC",
      0.01,
      bump,
    ).accounts({
      marketData: market_account
    }).rpc()
    console.log("tx:", tx, "market_account:", market_account.toBase58())
    const account = await program.account.market.fetch(market_account)
    console.log("market_account_data:", account.status)
    assert.strictEqual(account.vaultFull.toNumber(), 0);
  });

  it("test user account init", async () => {
    let [user_account, bump] = await PublicKey.findProgramAddress([USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()], program.programId)
    const tx = await program.methods.initializeUserAccount(
      bump,
    ).accounts({
      userAccount: user_account
    }).rpc()
    console.log("tx:", tx, "user_account:", user_account.toBase58())
    const account = await program.account.userAccount.fetch(user_account)
    console.log("user_account_data:", account)
    assert.strictEqual(account.positionSeedOffset.toNumber(), 0);
  });

  it("test deposit", async () => {
    console.log("test vault token account===>:", SPL.fromTokenAccount.address.toBase58())


    let [user_account, bump] = await PublicKey.findProgramAddress([USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()], program.programId)
    let [market_account, _bump] = await PublicKey.findProgramAddress([MARKET_ACCOUNT_SEED, encode("BTC")], program.programId)

    console.log(
      "\ntokenMint", SPL.mint.toBase58(),
      "\nuserTokenAccount:", SPL.userTokenAccount.address.toBase58(),
      "\nuserAccount:", user_account.toBase58(),
      "\nmarket_account:", market_account.toBase58()
    )

    const tx = await program.methods.deposit(new BN(1000000000), "BTC").accounts({
      tokenMint: SPL.mint,
      userTokenAccount: SPL.userTokenAccount.address,
      userAccount: user_account,
      vaultTokenAccount: new PublicKey("V5apPo62k57ap6h6c5uQnX2T6scJDaXap1dwhj7xomx"),
      marketAccount: market_account,
    }).signers([]).rpc()

    const account = await program.account.userAccount.fetch(user_account)
    console.log("user_account_data:", account)
    assert.strictEqual(account.positionSeedOffset.toNumber(), 0);

    let vault_account = await getAccount(provider.connection, new PublicKey("V5apPo62k57ap6h6c5uQnX2T6scJDaXap1dwhj7xomx"))
    assert.strictEqual(vault_account.amount, BigInt(51000000000))
    console.log("vault_account_amount:", vault_account.amount)
  });

});

