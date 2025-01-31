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

  let global, f44Vault: PublicKey;
  let f44Mint: PublicKey;
  let tokenAta: PublicKey;
  const BONDING_CURVE = "BONDING-CURVE";
  const SOL_VAULT_SEED = "SOL-VAULT-SEED";
  const VAULT_SEED = "VAULT-SEED";
  const tokenDecimal = 9;
  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  // please assume that you already mint spl token
  f44Mint = new PublicKey("CxgN5z1wdKavjszkmbgAwZrgVKVKinZpPYET2T3RVkGY");

  it("GET PDA", async () => {
    [global] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("GLOBAL-STATE-SEED")],
      program.programId
    );
    console.log("Get Global PDA->", global.toString());

    [f44Vault] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("F44-VAULT-SEED"),
        f44Mint.toBuffer()
      ],
      program.programId
    );
    console.log("Get f44Vault PDA->", f44Vault.toString());

  });

  it("Is initialized!", async () => {
    try {
      const tx = await program.rpc.initialize({
        accounts: {
          global,
          owner: owner.publicKey,
          f44Mint,
          f44Vault,
          systemProgram: SystemProgram.programId,
          tokenProgram:TOKEN_PROGRAM_ID
        },
        signers: [owner]
      });
      console.log("Initialize Tx->", tx);
    } catch (error) {
      console.log(error);
    }
  });
  /*
  it("set params", async () => {
    const initialVirtualTokenReserves = "1073000000000000";
    const initialVirtualSolReserves = "30000000000";
    const initialRealTokenReserves = "793100000000000";
    const tokenTotalSupply = "1000000000000000";
    const feeBasisPoints = 100; // 1% for buy & sell
    const mcap = "300000000000";
    const ownerWallet = new PublicKey(
      "2vKHp96ccuX6pP55o8mzCfRS7rD5Lz3gDWGQMwHjdEpF"
    );
    const createFee = 6900000; // 1sol (1sol = 10 ** 9 lamports) 0.0069 $1

    try {
      const tx = await program.rpc.setParams(
        feeRecipient,
        ownerWallet,
        new anchor.BN(initialVirtualTokenReserves),
        new anchor.BN(initialVirtualSolReserves),
        new anchor.BN(initialRealTokenReserves),
        new anchor.BN(tokenTotalSupply),
        new anchor.BN(mcap),
        new anchor.BN(feeBasisPoints),
        new anchor.BN(createFee),
        {
          accounts: {
            global,
            user: owner.publicKey
          },
          signers: [owner]
        }
      );
      const globalData = await program.account.global.fetch(global);
      console.log("globalData->", {
        initialized: globalData.initialized,
        authority: globalData.authority.toBase58(),
        feeRecipient: globalData.feeRecipient.toBase58(),
        ownerWallet: globalData.ownerWallet.toBase58(),
        initialVirtualTokenReserves: parseInt(globalData.initialVirtualTokenReserves.toString()),
        initialVirtualSolReserves: parseInt(globalData.initialVirtualSolReserves.toString()),
        initialRealTokenReserves: parseInt(globalData.initialRealTokenReserves.toString()),
        tokenTotalSupply: parseInt(globalData.tokenTotalSupply.toString()),
        feeBasisPoints: parseInt(globalData.feeBasisPoints.toString()),
        mcapLimit: parseInt(globalData.mcapLimit.toString()),
        createFee: parseInt(globalData.createFee.toString()),
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log(error);
    }
  });
  */
});
