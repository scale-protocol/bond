import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getAccount, transfer } from "@solana/spl-token";
import { PublicKey, } from '@solana/web3.js'
import { Bond } from "../target/types/bond";
const BN = anchor.BN;
const encode = anchor.utils.bytes.utf8.encode;
import { assert, expect } from 'chai';
import initSplAccounts from "./utils";
import * as buffer from "buffer";

const PAIR = "BTC/USD"

const VAULT_TOKEN_ACCOUNT_SEED = encode("scale_vault");

const VAULT_TOKEN_AUTHORITY_SEED = encode("scale_vault_authority");

const USER_ACCOUNT_SEED = encode("scale_user_account");

const MARKET_ACCOUNT_SEED = encode("scale_market_account");

const POSITION_INDEX_ACCOUNT_SEED = encode("scale_position_index_account");

const POSITION_ACCOUNT_SEED = encode("scale_position_account");

const VAULT_ACCOUNT = "AShVQFmFX1K74Ww1L2yaZRvmvUa6rAqmPWdoreN6Xato"

// pyth.network BTC
const BTC_MAIN_ACCOUNT = "GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU";
const BTC_TESTNET_ACCOUNT = "DJW6f4ZVqCnpYNN9rNuzqUcCvkVtBgixo8mq9FKSsCbJ";
const BTC_DEVNET_ACCOUNT = "HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J";
// pyth.network ETH
const ETH_MAIN_ACCOUNT = "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB";
const ETH_TESTNET_ACCOUNT = "7A98y76fcETLHnkCxjmnUrsuNrbUae7asy4TiVeGqLSs";
const ETH_DEVNET_ACCOUNT = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";

// chainlink BTC
const BTC_CHAINLINK_MAIN_ACCOUNT = "CGmWwBNsTRDENT5gmVZzRu38GnNnMm1K5C3sFiUUyYQX";
const BTC_CHAINLINK_DEVNET_ACCOUNT = "CzZQBrJCLqjXRfMjRN3fhbxur2QYHUzkpaRwkWsiPqbz";
// chainlink ETH
const ETH_CHAINLINK_MAIN_ACCOUNT = "5WyTBrEgvkAXjTdYTLY9PVrztjmz4edP5W9wks9KPFg5";
const ETH_CHAINLINK_DEVNET_ACCOUNT = "2ypeVyYnZaW2TNYXXTaZq9YhYvnqcjCiifW1C6n8b7Go";


describe("bond", () => {
  const provider = anchor.AnchorProvider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Bond as Program<Bond>;
  var SPL;

  // let mint = new PublicKey("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T");
  it("test vault account init", async () => {
    let SPL_ACCOUNT = await initSplAccounts({ provider: provider })
    SPL = SPL_ACCOUNT
    let [usdcVault, usdcBump] = await PublicKey.findProgramAddress(
      [
        VAULT_TOKEN_ACCOUNT_SEED,
        // USDC.toBuffer(),
      ],
      program.programId
    );
    // Add your test here.
    var tx = await program.methods.initializeVault(usdcBump).accounts({
      tokenMint: SPL_ACCOUNT.mint,
      vaultAccount: usdcVault,
    }).signers([]).rpc();
    let [vault_pda, pda_bump] = await PublicKey.findProgramAddress([VAULT_TOKEN_AUTHORITY_SEED], program.programId)
    let vault_account = await getAccount(provider.connection, usdcVault)

    console.log(
      "\nvault_account:", vault_account.address.toBase58(),
      "\nvault_account.mint:", vault_account.mint.toBase58(),
      "\nspl token mint:", SPL_ACCOUNT.mint.toBase58(),
      "\nvault_account_owner:", vault_account.owner.toBase58(),
      "\namount:", vault_account.amount,
      "\nvault_pda:", vault_pda.toBase58(),
      "\npda_bump:", pda_bump);

    assert.isTrue(vault_account.owner.equals(vault_pda));
    assert.isTrue(vault_account.mint.equals(SPL_ACCOUNT.mint));
    assert.isTrue(vault_account.address.equals(usdcVault));

    var xx = await transfer(
      provider.connection,
      SPL_ACCOUNT.fromWallet,
      SPL_ACCOUNT.fromTokenAccount.address,
      new PublicKey(VAULT_ACCOUNT),
      SPL_ACCOUNT.fromWallet.publicKey,
      50000000000
    );
    let vault_account_1 = await getAccount(provider.connection, usdcVault)

    assert.strictEqual(vault_account_1.amount, BigInt(50000000000));
    console.log("\ntest vault token account:", xx, "\namount:", vault_account_1.amount)
  });


  it("test market account init", async () => {
    let [market_account, bump] = await PublicKey.findProgramAddress([MARKET_ACCOUNT_SEED, encode(PAIR)], program.programId)
    var tx = await program.methods.initializeMarket(
      PAIR,
      0.01,
      bump,
      BTC_DEVNET_ACCOUNT,
      BTC_CHAINLINK_DEVNET_ACCOUNT,
    ).accounts({
      marketData: market_account
    }).rpc()
    console.log("tx:", tx, "market_account:", market_account.toBase58())
    var account = await program.account.market.fetch(market_account)
    console.log("market_account_data:", account.status)
    assert.strictEqual(account.vaultFull.toNumber(), 0);
  });

  it("test user account init", async () => {
    let [user_account, bump] = await PublicKey.findProgramAddress([USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()], program.programId)
    var tx = await program.methods.initializeUserAccount(
    ).accounts({
      userAccount: user_account,
    }).rpc()
    console.log("tx:", tx, "user_account:", user_account.toBase58())
    var account = await program.account.userAccount.fetch(user_account)
    console.log("user_account_data:", account)
    assert.strictEqual(account.positionSeedOffset.toString(), "1");
  });

  it("test deposit", async () => {
    console.log("token from token account", SPL.fromTokenAccount.address.toBase58())


    let [user_account, bump] = await PublicKey.findProgramAddress([USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()], program.programId)
    let [market_account, _bump] = await PublicKey.findProgramAddress([MARKET_ACCOUNT_SEED, encode(PAIR)], program.programId)

    console.log(
      "\ntokenMint", SPL.mint.toBase58(),
      "\nuserTokenAccount:", SPL.userTokenAccount.address.toBase58(),
      "\nuserAccount:", user_account.toBase58(),
      "\nmarket_account:", market_account.toBase58()
    )

    const tx = await program.methods.deposit(new BN(1000000000), PAIR).accounts({
      tokenMint: SPL.mint,
      userTokenAccount: SPL.userTokenAccount.address,
      userAccount: user_account,
      vaultTokenAccount: new PublicKey(VAULT_ACCOUNT),
      marketAccount: market_account,
    }).signers([]).rpc()

    const account = await program.account.userAccount.fetch(user_account)
    console.log("user_account_data:", account)
    assert.strictEqual(account.positionSeedOffset.toString(), "1");

    let vault_account = await getAccount(provider.connection, new PublicKey(VAULT_ACCOUNT))
    assert.strictEqual(vault_account.amount, BigInt(51000000000))
    console.log("vault_account_amount:", vault_account.amount)
  });

  it("test open position", async () => {
    // let [user_account, _a] = await PublicKey.findProgramAddress([USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()], program.programId)
    // let [market_account, _b] = await PublicKey.findProgramAddress([MARKET_ACCOUNT_SEED, encode(PAIR)], program.programId)
    // // let user_account_data = await getAccount(provider.connection, user_account)
    // // let user_account_data = program.account.userAccount.fetch(user_account)
    // let [position_account, _c] = await PublicKey.findProgramAddress(
    //   [
    //     POSITION_ACCOUNT_SEED,
    //     provider.wallet.publicKey.toBytes(),
    //     market_account.toBytes(),
    //     encode("1")
    //   ],
    //   program.programId,
    // )
    // let [position_index_account, _d] = await PublicKey.findProgramAddress(
    //   [
    //     POSITION_INDEX_ACCOUNT_SEED,
    //     provider.wallet.publicKey.toBytes(),
    //   ],
    //   program.programId,
    // )
    // var tx = await program.methods.openPosition(
    //   PAIR,
    //   20.3,
    //   new BN(4),
    //   1,
    //   1,
    // ).accounts({
    //   userAccount: user_account,
    //   marketAccount: market_account,
    //   positionAccount: position_account,
    //   // positionIndexAccount: position_index_account,
    //   pythPriceAccount: BTC_DEVNET_ACCOUNT,
    // }).rpc()
  });
});

