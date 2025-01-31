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

  let global, f44Vault, bondingCurve, associatedBondingCurve: PublicKey;
  let f44Mint, agentMint: PublicKey;
  let tokenAta: PublicKey;
  const BONDING_CURVE = "BONDING-CURVE";
  const SOL_VAULT_SEED = "SOL-VAULT-SEED";
  const VAULT_SEED = "VAULT-SEED";
  const tokenDecimal = 9;
  type Event = anchor.IdlEvents<(typeof program)["idl"]>;
  // please assume that you already mint spl token
  f44Mint = new PublicKey("CxgN5z1wdKavjszkmbgAwZrgVKVKinZpPYET2T3RVkGY");
  agentMint = new PublicKey("7XJMQgzNco7g58Fk6iEasf5SzEtgP9Rh63fDHHp2bPyV");

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

    [bondingCurve] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("BONDING-CURVE"),
        agentMint.toBuffer()
      ],
      program.programId
    );

    [associatedBondingCurve] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("VAULT-SEED"),
        agentMint.toBuffer()
      ],
      program.programId
    );
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
  it("set params", async () => {
    // const agentAmount = 1000000000;
    const agentAmount = 100000; // This value is only for testing. Please use the above value in product

    const ownerWallet = new PublicKey(
      "2vKHp96ccuX6pP55o8mzCfRS7rD5Lz3gDWGQMwHjdEpF"
    );
    const feeAmount = 1000; // The user should pay 1,000 F44 token when create the pool
    const createFee = 1000; // It is the same as feeAmount and it will be deleted in smart contract later cause we won't use that variable :(

    try {
      const tx = await program.rpc.setParams(
        feeRecipient,
        ownerWallet,
        new anchor.BN(agentAmount),
        new anchor.BN(feeAmount),
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
        f44Mint: globalData.f44Mint.toBase58(),
        f44Vault: globalData.f44Vault.toBase58(),
        f44Supply: parseInt(globalData.f44Supply.toString()),
        feeAmount: parseInt(globalData.feeAmount.toString()),
        agentAmount: parseInt(globalData.agentAmount.toString()),
        createFee: parseInt(globalData.createFee.toString())
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log(error);
    }
  });
  it("Deposit F44 tokens to the vault PDA controlled by the contract", async() => {
    try {
      const amount = 1000000 * (10 ** 6);
      const associatedOwnerAccount = await getAssociatedTokenAddress(
        f44Mint,
        owner.publicKey
      );

      const tx = await program.rpc.deposit(
        new anchor.BN(amount), {
          accounts: {
            global,
            owner: owner.publicKey,
            f44Mint,f44Vault,
            associatedOwnerAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
          },
          signers: [owner]
        }
      );
      console.log("Transaction was success and hash is ", tx);
    } catch (error) {
      console.log(error);
    }
  });
  it("Create the pool", async() => {
    try {
      const initialPrice = 0.01;
      const curveSlope = 0.00001;
      const globalData = await program.account.global.fetch(global);
      const amount = Number(globalData.agentAmount) * (10 ** 6);
      console.log(`Deposit amount is ${Number(globalData.agentAmount)} and decimals is 6 So amount param is ${amount}`);
      const associatedUserAccount = await getAssociatedTokenAddress(
        agentMint,
        user.publicKey
      );
      const associatedUserF44Account = await getAssociatedTokenAddress(
        f44Mint,
        user.publicKey
      );

      const tx = await program.rpc.create(
        initialPrice,
        curveSlope,
        new anchor.BN(amount), {
          accounts: {
            user: user.publicKey,
            mint: agentMint,
            bondingCurve,
            associatedBondingCurve,
            associatedUserAccount,
            f44Mint,
            f44Vault,
            associatedUserF44Account,
            global,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
          },
          signers: [user]
        }
      );
      console.log("Create Pool tx hash is ", tx);
    } catch (error) {
      console.log(error);
    }
  });
  it("Buy agent Token with F44 token", async() => {
    try {
      const f44Amount = 10;
      const amount = await calculateBuyF44Cost(f44Amount);
      console.log("The agent token amount that we can buy with f44 Amount is ", amount);
      const slippage = 1; //1%
      const maxF44Amount = f44Amount * (100 + slippage) / 100 * 10 ** 6;
      console.log("max f44 amount is ", maxF44Amount);
      const associatedUser = await getAssociatedTokenAddress(
        agentMint,
        buyer.publicKey
      );
      const associatedUserF44Account = await getAssociatedTokenAddress(
        f44Mint,
        buyer.publicKey
      );

      const tx = await program.rpc.buy(
        new anchor.BN(parseInt((amount * 10 ** 6).toString())),
        new anchor.BN(parseInt(maxF44Amount.toString())), {
          accounts: {
            global,
            mint: agentMint,
            bondingCurve,
            associatedBondingCurve,
            associatedUser,
            f44Mint,
            f44Vault,associatedUserF44Account,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [buyer]
        }
      );
      console.log("Buy tx hash is ", tx);
    } catch (error) {
      console.log(error);
    }
  });
  it("Sell agent token", async() => {
    try {
      const tokenAmount = 700;
      const amount = await calculateF44SellCost(tokenAmount);
      console.log("The agent token amount that we can buy with f44 Amount is ", amount);
      const slippage = 1; //1%
      const minF44Amount = amount * (100 - slippage) / 100 * 10 ** 6;

      const associatedUser = await getAssociatedTokenAddress(
        agentMint,
        buyer.publicKey
      );
      const associatedUserF44Account = await getAssociatedTokenAddress(
        f44Mint,
        buyer.publicKey
      );

      const tx = await program.rpc.sell(
        new anchor.BN(parseInt((tokenAmount * 10 ** 6).toString())),
        new anchor.BN(parseInt(minF44Amount.toString())), {
          accounts: {
            global,
            mint: agentMint,
            bondingCurve,
            associatedBondingCurve,
            associatedUser,
            f44Mint,
            f44Vault,
            associatedUserF44Account,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            clock: SYSVAR_CLOCK_PUBKEY
          },
          signers: [buyer]
        }
      );
      console.log("Sell tx hash is ", tx);
    } catch (error) {
      console.log(error);
    }
  });
  it("Withdraw agent token and f44 tokens", async() => {
    try {
      const associatedUser = await getAssociatedTokenAddress(
        agentMint,
        owner.publicKey
      );
      const associatedUserF44Account = await getAssociatedTokenAddress(
        f44Mint,
        owner.publicKey
      );
      const tx = await program.rpc.withdraw({
        accounts: {
          global,
          mint: agentMint,
          bondingCurve,
          associatedBondingCurve,
          associatedUser,
          f44Mint,
          f44Vault,
          associatedUserF44Account,
          ownerWallet: owner.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
        },
        signers: [owner]
      })
    } catch (error) {
      console.log(error);
    }
  })

  async function calculateBuyF44Cost(f44Amount: number) {
    try {
      const bondingCurveData = await program.account.bondingCurve.fetch(bondingCurve);
      console.log("bondingCurveData->", bondingCurveData);
      const initialPrice = parseFloat(bondingCurveData.initialPrice.toString());
      const curveSlope = parseFloat(bondingCurveData.curveSlope.toString());
      const tokenReserves = parseFloat(bondingCurveData.tokenReserves.toString());
      const A = curveSlope / 2;
      const B = curveSlope * tokenReserves + initialPrice;
      const C = -f44Amount;

      const x = (-B + Math.sqrt(Math.pow(B,2) - 4 * A * C)) / (2 * A);
      return x;
    } catch (error) {
      console.log(error);
    }
  }

  async function calculateF44SellCost(tokenAmount: number) {
    try {
      const bondingCurveData = await program.account.bondingCurve.fetch(bondingCurve);
      console.log("bondingCurveData->", bondingCurveData);
      const initialPrice = parseFloat(bondingCurveData.initialPrice.toString());
      const curveSlope = parseFloat(bondingCurveData.curveSlope.toString());
      const tokenReserves = parseFloat(bondingCurveData.tokenReserves.toString());
    
      let firstPrice = initialPrice + curveSlope * tokenReserves;
      let lastPrice = initialPrice + curveSlope * (tokenReserves - tokenAmount);
      console.log("average price is ", (firstPrice + lastPrice ) / 2 );
      let x = (firstPrice + lastPrice) / 2 * tokenAmount;
      console.log("Sell F44 amount is ", x );

      return x;
    } catch (error) {
      console.log(error);
    }
  }

});
