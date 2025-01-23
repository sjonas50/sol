import { percentAmount, generateSigner, signerIdentity, createSignerFromKeypair } from '@metaplex-foundation/umi'
import { TokenStandard, createAndMint } from '@metaplex-foundation/mpl-token-metadata'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { mplCandyMachine } from "@metaplex-foundation/mpl-candy-machine";
import { Keypair, PublicKey } from '@solana/web3.js';
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';
// import "@solana/web3.js";
import { Signer } from "@metaplex-foundation/umi";

const umi = createUmi('https://api.devnet.solana.com'); //Replace with your QuickNode RPC Endpoint

const secret =[113,63,93,213,68,178,22,189,136,49,33,174,196,213,238,242,164,106,9,180,15,3,238,80,159,127,118,18,231,206,240,93,21,168,99,61,85,242,222,187,12,44,91,158,122,83,103,113,125,136,28,83,108,248,78,219,197,250,38,187,70,109,130,194];
const userWallet = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(secret));
const userWalletSigner = createSignerFromKeypair(umi, userWallet);

const F44Agent = {
    name: "F44 AI Test",
    symbol: "F44.ai",
    uri: "https://ipfs.io/ipfs/bafkreibwclebyzx27s5tyxvutwoc2t4t3bekdgqna3bdtj76biqsenq37y",
};

// function generateKeypairWithSuffix(suffix: string) {
//     let mintKey: Keypair;
//     while (true) {
//         const keypair = Keypair.generate();
//         const publicKey = keypair.publicKey.toBase58(); // Convert public key to Base58 string

//         if (publicKey.endsWith(suffix)) {
//             mintKey = keypair;
//             console.log("token address:", mintKey.publicKey.toString());
//             break;
//         }
//     }
//     return mintKey;
// }

const mint = generateSigner(umi);

umi.use(signerIdentity(userWalletSigner));
umi.use(mplCandyMachine())

function mintF44Agent() {
    createAndMint(umi, {
        mint,
        authority: umi.identity,
        name: F44Agent.name,
        symbol: F44Agent.symbol,
        uri: F44Agent.uri,
        sellerFeeBasisPoints: percentAmount(0),
        decimals: 6,
        amount:  1000000000_000000,
        tokenOwner: userWallet.publicKey,
        tokenStandard: TokenStandard.Fungible,
    }).sendAndConfirm(umi).then(() => {
        console.log("Successfully minted 1 million tokens (", mint.publicKey, ")");
    }).catch((error) => {
        console.error("Error minting tokens:", error);
        new Promise(resolve => setTimeout(resolve, 2000));
        mintF44Agent()
    });
}

mintF44Agent()