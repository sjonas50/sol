use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token,TokenAccount,Transfer, transfer};

use crate::{
    state::{Global, BondingCurve},
    constants::{GLOBAL_STATE_SEED, BONDING_CURVE, SOL_VAULT_SEED},
    error::*,
    events::*,
};
use solana_program::{program::invoke_signed, system_instruction};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SOL_VAULT_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: this should be set by admin
    pub vault: UncheckedAccount<'info>,


    #[account(
        mut,
        seeds = [BONDING_CURVE, mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user
    )]
    pub associated_user: Box<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn sell(ctx: Context<Sell>, amount: u64, min_sol_output: u64) -> Result<()> {
    let accts = ctx.accounts;
    require!(accts.fee_recipient.key() == accts.global.fee_recipient, F44Code::UnValidFeeRecipient);
    require!(accts.bonding_curve.complete == false, F44Code::BondingCurveComplete);
    require!(amount >0 , F44Code::ZeroAmount);

    let bonding_curve = &accts.bonding_curve;

    // Calculate the required SOL cost for the given token amount
    let sol_cost = calculate_sol_cost(bonding_curve, amount)?;

    // Ensure the SOL cost does not exceed max_sol_cost
    require!(sol_cost >= min_sol_output, F44Code::TooLittleSolReceived);

    // send sol from vault account to user (calculate fee)
    let binding = accts.mint.key();

    let (_, bump) = Pubkey::find_program_address(&[SOL_VAULT_SEED, binding.as_ref()], ctx.program_id);
    let vault_seeds = &[SOL_VAULT_SEED, binding.as_ref(), &[bump]];
    let signer = &[&vault_seeds[..]];

    let fee_amount = accts.global.fee_basis_points * sol_cost / 10000;

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.fee_recipient.key(), fee_amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.fee_recipient.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        signer,
    )?;

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.user.key(), sol_cost - fee_amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.user.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        signer,
    )?;
    // send tokens to the vault
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_user.to_account_info().clone(),
            to: accts.associated_bonding_curve.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;

    //  update the bonding curve
    accts.bonding_curve.real_token_reserves += amount;
    accts.bonding_curve.virtual_token_reserves += amount;
    accts.bonding_curve.virtual_sol_reserves -= sol_cost - fee_amount;
    accts.bonding_curve.real_sol_reserves -= sol_cost - fee_amount;

    // Log the TradeEvent details

     msg!(
        "Trade // Type: Sell, User: {}, Mint: {}, BondingCurve: {}, Timestamp: {}, SolCost: {}, Amount: {}, IsBuy: {}, VirtualSolReserves: {}, VirtualTokenReserves: {}",
        accts.user.key(),
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.clock.unix_timestamp,
        sol_cost,
        amount,
        false,
        accts.bonding_curve.virtual_sol_reserves,
        accts.bonding_curve.virtual_token_reserves
    );

    emit!(
        TradeEvent { 
            mint: accts.mint.key(), 
            sol_amount: sol_cost, 
            token_amount: amount, 
            is_buy: false, 
            user: accts.user.key(), 
            timestamp: accts.clock.unix_timestamp, 
            virtual_sol_reserves: accts.bonding_curve.virtual_sol_reserves, 
            virtual_token_reserves: accts.bonding_curve.virtual_token_reserves, 
        }
    );

    Ok(())
}

fn calculate_sol_cost(bonding_curve: &Account<BondingCurve>, token_amount: u64) -> Result<u64> {
    let sol_cost = ((token_amount as u128).checked_mul(bonding_curve.virtual_sol_reserves as u128).ok_or(F44Code::MathOverflow)?.checked_div(bonding_curve.virtual_token_reserves as u128).ok_or(F44Code::MathOverflow)?) as u64;

    Ok(sol_cost as u64)
}