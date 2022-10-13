import { mintTo, getOrCreateAssociatedTokenAccount, transfer, createMint, getAccount } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, Connection, Signer } from '@solana/web3.js'
import * as buffer from 'buffer';
import * as fs from 'fs';
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

function loadLocalTokenWallet(file: string): Keypair {
    console.log("local token wallet path:", file)
    var wallet: Keypair;
    try {
        let data = fs.readFileSync(file)
        wallet = Keypair.fromSecretKey(buffer.Buffer.from(JSON.parse(data.toString())))
    } catch (e) {
        wallet = Keypair.generate();
        fs.writeFile(
            file,
            `[${wallet.secretKey.toString()}]`,
            { flag: 'a' },
            function (err) {
                throw err;
            }
        )
    }
    return wallet;
}

async function loadLocalTokenMint(
    file: string,
    connection: Connection,
    payer: Signer,
    mintAuthority: PublicKey,
    freezeAuthority: PublicKey | null,
    decimals: number,): Promise<PublicKey> {
    console.log("local token mint path:", file)
    var mint: PublicKey;
    try {
        let data = fs.readFileSync(file).toString()
        data = data.trim()
        mint = new PublicKey(data)
    } catch (error) {
        mint = await createMint(connection, payer, mintAuthority, freezeAuthority, decimals);
        fs.writeFile(file, mint.toBase58(), { flag: 'a' }, (err) => {
            console.log(err)
        })
        console.log(error)
    }
    return mint;
}
async function loadLocalTokenAddress(
    file: string,
    connection: Connection,
    payer: Signer,
    mint: PublicKey,
    owner: PublicKey
): Promise<PublicKey> {
    console.log("local token mint path:", file)
    var user_token_account: PublicKey;
    try {
        let data = fs.readFileSync(file).toString()
        data = data.trim()
        user_token_account = new PublicKey(data)
    } catch (error) {
        let create = await getOrCreateAssociatedTokenAccount(connection, payer, mint, owner);
        user_token_account = create.address
        fs.writeFile(file, user_token_account.toBase58(), { flag: 'a' }, (err) => {
            console.log(err)
        })
        console.log(error)
    }
    return user_token_account;
}