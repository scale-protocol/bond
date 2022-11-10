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

const PAIR = {
  BTC: "BTC/USD",
  ETH: "ETH/USD",
  SOL: "SOL/USD",
}

const VAULT_TOKEN_ACCOUNT_SEED = encode("scale_vault");

const VAULT_TOKEN_AUTHORITY_SEED = encode("scale_vault_authority");

const USER_ACCOUNT_SEED = encode("scale_user_account");

const MARKET_ACCOUNT_SEED = encode("scale_market_account");

const POSITION_ACCOUNT_SEED = encode("scale_position_account");

const VAULT_ACCOUNT = "F7NPLGunbG5rmKnYY7opt1SWfgNXRg8LUyoi4LK7wpu4"



const PYTH_PRICE = {
  MAINNETBETA: {
    BTC: "GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU",
    ETH: "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB",
    SOL: "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",
  },
  TESTNET: {
    BTC: "DJW6f4ZVqCnpYNN9rNuzqUcCvkVtBgixo8mq9FKSsCbJ",
    ETH: "7A98y76fcETLHnkCxjmnUrsuNrbUae7asy4TiVeGqLSs",
    SOL: "7VJsBtJzgTftYzEeooSDYyjKXvYRWJHdwvbwfBvTg9K",
  },
  DEVNET: {
    BTC: "HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J",
    ETH: "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw",
    SOL: "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix",
  },
}

const CHAINLINK_PRICE = {
  MAINNETBETA: {
    BTC: "CGmWwBNsTRDENT5gmVZzRu38GnNnMm1K5C3sFiUUyYQX",
    ETH: "5WyTBrEgvkAXjTdYTLY9PVrztjmz4edP5W9wks9KPFg5",
    SOL: "CcPVS9bqyXbD9cLnTbhhHazLsrua8QMFUHTutPtjyDzq",
  },
  TESTNET: {
    BTC: "",
    ETH: "",
    SOL: "",
  },
  DEVNET: {
    BTC: "CzZQBrJCLqjXRfMjRN3fhbxur2QYHUzkpaRwkWsiPqbz",
    ETH: "2ypeVyYnZaW2TNYXXTaZq9YhYvnqcjCiifW1C6n8b7Go",
    SOL: "HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6",
  },
}

const MARKET_ACCOUNT = {
  BTC: "",
  ETH: "",
  SOL: "",
}
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

    // var xx = await transfer(
    //   provider.connection,
    //   SPL_ACCOUNT.fromWallet,
    //   SPL_ACCOUNT.fromTokenAccount.address,
    //   new PublicKey(VAULT_ACCOUNT),
    //   SPL_ACCOUNT.fromWallet.publicKey,
    //   5000
    // );
    // let vault_account_1 = await getAccount(provider.connection, usdcVault)

    // assert.strictEqual(vault_account_1.amount, BigInt(5000));
    // console.log("\ntest vault token account:", xx, "\namount:", vault_account_1.amount)
  });


  it("test market account init", async () => {
    let [market_account_btc, bump] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED,
        encode(PAIR.BTC)],
      program.programId)
    var tx = await program.methods.initializeMarket(
      PAIR.BTC,
      0.01,
      bump,
      PYTH_PRICE.DEVNET.BTC,
      CHAINLINK_PRICE.DEVNET.BTC,
    ).accounts({
      marketAccount: market_account_btc
    }).rpc()
    console.log("tx:", tx, "market_account_btc:", market_account_btc.toBase58())
    var account = await program.account.market.fetch(market_account_btc)
    console.log("market_account_data:", account.status)
    assert.strictEqual(account.vaultFull.toNumber(), 0);
    // --- init market----
    let [market_account_eth, e_bump] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED,
        encode(PAIR.ETH)],
      program.programId)
    var tx = await program.methods.initializeMarket(
      PAIR.ETH,
      0.01,
      e_bump,
      PYTH_PRICE.DEVNET.ETH,
      CHAINLINK_PRICE.DEVNET.ETH,
    ).accounts({
      marketAccount: market_account_eth
    }).rpc()
    console.log("tx:", tx, "market_account_eth:", market_account_eth.toBase58())

    let [market_account_sol, s_bump] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED,
        encode(PAIR.SOL)],
      program.programId)
    var tx = await program.methods.initializeMarket(
      PAIR.SOL,
      0.01,
      s_bump,
      PYTH_PRICE.DEVNET.SOL,
      CHAINLINK_PRICE.DEVNET.SOL,
    ).accounts({
      marketAccount: market_account_sol
    }).rpc()
    console.log("tx:", tx, "market_account_sol:", market_account_sol.toBase58())
  });

  it("test market account investment", async () => {
    let [market_account, _bump] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED, encode(PAIR.BTC)],
      program.programId)
    var tx = await program.methods.investment(
      PAIR.BTC,
      new BN(10000)
    ).accounts({
      tokenMint: SPL.mint,
      userTokenAccount: SPL.userTokenAccount.address,
      vaultTokenAccount: new PublicKey(VAULT_ACCOUNT),
      marketAccount: market_account,
    }).rpc()
    const account = await getAccount(provider.connection, SPL.userTokenAccount.address);
    console.log("user_token_account amount:", account.amount);
    assert.strictEqual(account.amount, BigInt(40000));

    let vault_account_data = await getAccount(provider.connection, new PublicKey(VAULT_ACCOUNT));
    assert.strictEqual(vault_account_data.amount, BigInt(10000));
    console.log("vault_account_amount:", vault_account_data.amount)
  });

  it("test market account divestment", async () => {
    let [market_account, _bump] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED, encode(PAIR.BTC)],
      program.programId)

    let [vault_pda, _pda_bump] = await PublicKey.findProgramAddress([VAULT_TOKEN_AUTHORITY_SEED], program.programId)
    var tx = await program.methods.divestment(
      PAIR.BTC,
      new BN(1000)
    ).accounts({
      tokenMint: SPL.mint,
      userTokenAccount: SPL.userTokenAccount.address,
      vaultTokenAccount: new PublicKey(VAULT_ACCOUNT),
      pdaAuthorityAccount: vault_pda,
      marketAccount: market_account,
    }).rpc()
    const account = await getAccount(provider.connection, SPL.userTokenAccount.address)
    console.log("user_token_account amount:", account.amount)
    assert.strictEqual(account.amount, BigInt(40000 + 1000));

    let vault_account_data = await getAccount(provider.connection, new PublicKey(VAULT_ACCOUNT))
    assert.strictEqual(vault_account_data.amount, BigInt(10000 - 1000))
    console.log("vault_account_amount:", vault_account_data.amount)
  });

  it("test user account init", async () => {
    let [user_account, bump] = await PublicKey.findProgramAddress(
      [USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()],
      program.programId)
    var tx = await program.methods.initializeUserAccount(
      bump
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


    let [user_account, bump] = await PublicKey.findProgramAddress(
      [USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()],
      program.programId)


    console.log(
      "\ntokenMint", SPL.mint.toBase58(),
      "\nuserTokenAccount:", SPL.userTokenAccount.address.toBase58(),
      "\nuserAccount:", user_account.toBase58(),
    )

    const tx = await program.methods.deposit(new BN(1000)).accounts({
      tokenMint: SPL.mint,
      userTokenAccount: SPL.userTokenAccount.address,
      userAccount: user_account,
      vaultTokenAccount: new PublicKey(VAULT_ACCOUNT),
    }).signers([]).rpc()

    const account = await program.account.userAccount.fetch(user_account)
    console.log("user_account_data:", account)
    assert.strictEqual(account.positionSeedOffset.toString(), "1");

    let vault_account = await getAccount(provider.connection, new PublicKey(VAULT_ACCOUNT))
    assert.strictEqual(vault_account.amount, BigInt(10000))
    console.log("vault_account_amount:", vault_account.amount)
  });

  it("test open position", async () => {
    let [user_account, _a] = await PublicKey.findProgramAddress(
      [USER_ACCOUNT_SEED, provider.wallet.publicKey.toBytes()],
      program.programId)
    let [market_account_btc, _b] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED, encode(PAIR.BTC)],
      program.programId)
    let [market_account_eth, _e] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED, encode(PAIR.ETH)],
      program.programId)
    let [market_account_sol, _s] = await PublicKey.findProgramAddress(
      [MARKET_ACCOUNT_SEED, encode(PAIR.SOL)],
      program.programId)
    let [position_account, _c] = await PublicKey.findProgramAddress(
      [
        POSITION_ACCOUNT_SEED,
        provider.wallet.publicKey.toBytes(),
        user_account.toBytes(),
        encode("1")
      ],
      program.programId,
    )
    var tx = await program.methods.openPosition(
      PAIR.BTC,
      2.3,
      4,
      1,
      1,
    ).accounts({
      userAccount: user_account,
      marketAccount: market_account_btc,
      positionAccount: position_account,
      pythPriceAccount: PYTH_PRICE.DEVNET.BTC,
      chianlinkPriceAccount: CHAINLINK_PRICE.DEVNET.BTC,
      marketAccountBtc: market_account_btc,
      marketAccountEth: market_account_eth,
      marketAccountSol: market_account_sol,
      pythPriceAccountBtc: PYTH_PRICE.DEVNET.BTC,
      pythPriceAccountEth: PYTH_PRICE.DEVNET.ETH,
      pythPriceAccountSol: PYTH_PRICE.DEVNET.SOL,
      chainlinkPriceAccountBtc: CHAINLINK_PRICE.DEVNET.BTC,
      chainlinkPriceAccountEth: CHAINLINK_PRICE.DEVNET.ETH,
      chainlinkPriceAccountSol: CHAINLINK_PRICE.DEVNET.SOL,
    }).rpc()
  });
});

