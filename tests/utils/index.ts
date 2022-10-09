import { mintTo, getOrCreateAssociatedTokenAccount, transfer, createMint, getAccount } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js'

export default async function initSplAccounts({ provider }) {
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
        10000000000000
    );
    console.log('mint tx:', signature);
    signature = await transfer(
        connection,
        fromWallet,
        fromTokenAccount.address,
        toTokenAccount.address,
        fromWallet.publicKey,
        50000000000
    );
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(connection, fromWallet, mint, provider.wallet.publicKey);
    signature = await transfer(
        connection,
        fromWallet,
        fromTokenAccount.address,
        userTokenAccount.address,
        fromWallet.publicKey,
        50000000000
    );

    return {
        fromWallet: fromWallet,
        toWallet: toWallet,
        mint: mint,
        fromTokenAccount: fromTokenAccount,
        toTokenAccount: toTokenAccount,
        userTokenAccount: userTokenAccount,
    };
}