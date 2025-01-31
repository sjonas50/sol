use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint,Token,TokenAccount,Transfer, transfer}
};

use crate::{
    constants::{GLOBAL_STATE_SEED, BONDING_CURVE, F44_VAULT_SEED}, 
    state::{Global, BondingCurve},
    error::*,
    events::*,
};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,  // the mint address of token

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
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub associated_user: Box<Account<'info, TokenAccount>>,

    pub f44_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub associated_user_f44_account: Box<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock:  Sysvar<'info, Clock>,
}
// amount is the agent token amount
// max_f44_amount is the max f44 token amount that will be used as payment
pub fn buy(ctx: Context<Buy>, amount: u64, max_f44_amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(amount >0 , F44Code::ZeroAmount);
    require!(accts.bonding_curve.complete == false, F44Code::BondingCurveComplete);

    let bonding_curve = &accts.bonding_curve;
    let decimals = accts.mint.decimals;
    let f44_decimals = accts.f44_mint.decimals;

    // Calculate the required SOL cost for the given token amount
    let f44_amount = calculate_f44_cost(bonding_curve, amount as f64 / 10_u64.pow(decimals.into()) as f64)?;

    // Ensure the F44 Amount does not exceed max_f44_cost
    require!(f44_amount as u64 * 10_u64.pow(f44_decimals.into()) <= max_f44_amount, F44Code::TooMuchF44Required);

    // send f44 token to the f44 reserve pool
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_user_f44_account.to_account_info().clone(),
            to: accts.f44_vault.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, f44_amount as u64 * 10_u64.pow(f44_decimals.into()))?;

    accts.global.f44_supply += f44_amount as u64;
   
    // send token from agent token vault account to user
    let binding = accts.mint.key();

    let (_, bump) = Pubkey::find_program_address(&[BONDING_CURVE, binding.as_ref()], ctx.program_id);
    let vault_seeds = &[BONDING_CURVE, binding.as_ref(), &[bump]];
    let signer = &[&vault_seeds[..]];

    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_bonding_curve.to_account_info().clone(),
            to: accts.associated_user.to_account_info().clone(),
            authority: accts.bonding_curve.to_account_info().clone(),
        },
    );
    transfer(
        cpi_ctx.with_signer(signer),
        amount,
    )?;

    //  update the bonding curve
    accts.bonding_curve.token_reserves += (amount / 10_u64.pow(decimals.into())) as f64;

    let macp = (accts.bonding_curve.curve_slope as f64 * accts.bonding_curve.token_reserves as f64 + accts.bonding_curve.initial_price) * accts.bonding_curve.token_total_supply;
    msg!("Current market cap is {}", macp);

    if macp > accts.bonding_curve.mcap_limit {

        accts.bonding_curve.complete = true;

        msg!(
            "Bonding Curve Complete : User: {}, Mint: {}, BondingCurve: {}, Timestamp: {}",
            accts.user.key(),
            accts.mint.key(),
            accts.bonding_curve.key(),
            accts.clock.unix_timestamp
        );

        emit!(
            CompleteEvent { 
                user: accts.user.key(), 
                mint: accts.mint.key(), 
                bonding_curve: accts.bonding_curve.key(),
                timestamp: accts.clock.unix_timestamp, 
            }
        );
    } 

    msg!(
        "Trade // Type: Buy, User: {}, Mint: {}, BondingCurve: {}, Timestamp: {}, f44 Amount: {}, Amount: {}, IsBuy: {}, token_reserves: {}",
        accts.user.key(),
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.clock.unix_timestamp,
        f44_amount,
        amount,
        true,
        accts.bonding_curve.token_reserves
    );

    emit!(
        TradeEvent { 
            mint: accts.mint.key(), 
            amount: f44_amount as u64, 
            token_amount: (amount / 10_u64.pow(decimals.into())) as f64, 
            is_buy: true, 
            user: accts.user.key(), 
            timestamp: accts.clock.unix_timestamp, 
            token_reserves: accts.bonding_curve.token_reserves, 
        }
    );

    Ok(())
}

fn calculate_f44_cost(bonding_curve: &Account<BondingCurve>, token_amount: f64) -> Result<f64> {
    let first_price = bonding_curve.curve_slope * bonding_curve.token_reserves as f64 + bonding_curve.initial_price;
    let last_price = bonding_curve.curve_slope * (bonding_curve.token_reserves as f64 + token_amount) + bonding_curve.initial_price;
    msg!("The first price is {} and last price is {}", first_price.clone(), last_price.clone());
    let average_price = (first_price + last_price) / 2.0;
    msg!("The average price of token is {}", average_price);

    let f44_amount = token_amount * average_price;
    msg!("The F44 amount is {}", f44_amount);

    Ok(f44_amount)
}