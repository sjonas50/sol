import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { F44 } from "../target/types/f44";
import {
  TOKEN_PROGRAM_ID,
  createAccount,
  createInitializeMintInstruction,
  MINT_SIZE,
  getMinimumBalanceForRentExemptMint,
  createMint,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  mintTo,
  mintToChecked,
  getAccount,
  getMint,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  createMintToCheckedInstruction,
  getOrCreateAssociatedTokenAccount
} from "@solana/spl-token";

import * as bs58 from "bs58";
import {
  SystemProgram,
  Keypair,
  PublicKey,
  Transaction,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
  Connection,
  clusterApiUrl,
  sendAndConfirmTransaction
} from "@solana/web3.js";
import assert from "assert";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

describe("f44", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.F44 as Program<F44>;

  // Defind the constants for test
  const feeRecipient = new PublicKey(
    "8kiRRQHLcT6iRG9H3cyathqtSKhc8o7rp31wto2mvHBT"
  );

  const owner = Keypair.fromSecretKey(
    bs58.decode("KJ7yZn5AQchXPE5i74FsG1WNtccSQXgGUDtXJdyoyRa8hqSbdFH8R9NZiZnosKJnQqnRgSYvVCZqu3VqHaqF8GP")
  );

  const user = Keypair.fromSecretKey(
    bs58.decode(
      "2LU9Gir9pDVEsUWrRHLUUdPaVM642EmMGubgyZg2LNYk1uyD4LNRR5HshCENmfTUD3nPMeN7FCJKxEdu48YSEpta"
    )
  );

  const buyer = Keypair.fromSecretKey(
    bs58.decode(
      "TGW9dbYndwDA5VbBBsA3AQsGtTgoCetjpJwbuCjNF3pv2J1rCXraZNrNXHhu2fxKTaNCFiotT9z3QCnujQ3WGhD"
    )
  );

  let global: PublicKey;
  let globalBump: number;
  let mint: PublicKey;
  let tokenAta: PublicKey;
  const BONDING_CURVE = "BONDING-CURVE";
  const SOL_VAULT_SEED = "SOL-VAULT-SEED";
  const VAULT_SEED = "VAULT-SEED";
  const tokenDecimal = 9;
  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  // please assume that you already mint spl token
  mint = new PublicKey("CxgN5z1wdKavjszkmbgAwZrgVKVKinZpPYET2T3RVkGY");

  it("GET PDA", async () => {
    [global, globalBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("GLOBAL-STATE-SEED")],
      program.programId
    );
    console.log("Get Global PDA->", global.toString());
  });

  it("Is initialized!", async () => {
    try {
      /* Please ignore this step as the airdrop may not work.
      // 1 - Request Airdrop
      const signature = await program.provider.connection.requestAirdrop(
        owner.publicKey,
        10 ** 9
      );
       // 2 - Fetch the latest blockhash
      const { blockhash, lastValidBlockHeight } = await program.provider.connection.getLatestBlockhash();
      // 3 - Confirm transaction success
      await program.provider.connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature
      }, 'confirmed');
      */

      const tx = await program.rpc.initialize({
        accounts: {
          global,
          owner: owner.publicKey,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      });
      console.log("Initialize Tx->", tx);
    } catch (error) {
      console.log(error);
    }
  });
});
