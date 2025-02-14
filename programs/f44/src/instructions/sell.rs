use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    constants::{BONDING_CURVE, F44_VAULT_SEED, GLOBAL_STATE_SEED},
    error::*,
    events::*,
    state::{BondingCurve, Global},
};

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

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
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub associated_user: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub f44_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = f44_mint,
        associated_token::authority = user
    )]
    pub associated_user_f44_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn sell(ctx: Context<Sell>, amount: u64, min_f44_output: u64) -> Result<()> {
    let accts = ctx.accounts;
    let bonding_curve = &accts.bonding_curve;
    let decimals = accts.mint.decimals;
    let f44_decimals = accts.f44_mint.decimals;

    require!(
        accts.bonding_curve.complete == false,
        F44Code::BondingCurveComplete
    );
    require!(amount > 0, F44Code::ZeroAmount);
    require!(
        bonding_curve.token_reserves >= amount as f64 / 10_u64.pow(decimals.into()) as f64,
        F44Code::NotEnoughAmount
    );

    // Calculate the required SOL cost for the given token amount
    let token_amount = amount as f64 / 10_u64.pow(decimals.into()) as f64;
    let f44_amount = calculate_f44_cost(bonding_curve, token_amount)?;
    accts.bonding_curve.current_price = bonding_curve.curve_slope
        * (bonding_curve.token_reserves as f64 - token_amount)
        + bonding_curve.initial_price;

    // Ensure the SOL cost does not exceed max_sol_cost
    require!(
        f44_amount * 10_u64.pow(f44_decimals.into()) as f64 >= min_f44_output as f64,
        F44Code::TooLittleF44Received
    );

    // send f44 token from pool reserve to user
    let (_, bump) = Pubkey::find_program_address(&[GLOBAL_STATE_SEED], ctx.program_id);
    let global_seeds = &[GLOBAL_STATE_SEED, &[bump]];
    let signer = &[&global_seeds[..]];

    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.f44_vault.to_account_info().clone(),
            to: accts.associated_user_f44_account.to_account_info().clone(),
            authority: accts.global.to_account_info().clone(),
        },
    );
    transfer(
        cpi_ctx.with_signer(signer),
        (f44_amount * 10_u64.pow(f44_decimals.into()) as f64) as u64,
    )?;
    accts.global.f44_supply -= (f44_amount * 10_u64.pow(f44_decimals.into()) as f64) as u64;

    // send tokens to the vault
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Burn {
            mint: accts.mint.to_account_info().clone(),
            from: accts.associated_user.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    burn(cpi_ctx, amount)?;
    // burn agent tokens

    //  update the bonding curve
    let decimals = accts.mint.decimals;
    accts.bonding_curve.token_reserves -= amount as f64 / 10_u64.pow(decimals.into()) as f64;

    // Calculate market cap
    let macp = (accts.bonding_curve.curve_slope * accts.bonding_curve.token_reserves
        + accts.bonding_curve.initial_price)
        * accts.bonding_curve.token_total_supply;
    accts.bonding_curve.current_mcap = macp;

    // Log the TradeEvent details

    msg!(
        "TradeEvent - type: Sell, user: {}, mint: {}, bondingCurve: {}, timestamp: {}, f44Amount: {}, amount: {}, isBuy: {}, tokenReserves: {}, currentPrice: {}, currentMcap: {}",
        accts.user.key(),
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.clock.unix_timestamp,
        f44_amount,
        amount,
        false,
        accts.bonding_curve.token_reserves,
        accts.bonding_curve.current_price,
        accts.bonding_curve.current_mcap,
    );

    emit!(TradeEvent {
        mint: accts.mint.key(),
        amount: f44_amount as u64,
        token_amount: amount,
        is_buy: false,
        user: accts.user.key(),
        timestamp: accts.clock.unix_timestamp,
        token_reserves: accts.bonding_curve.token_reserves,
        last_price: accts.bonding_curve.current_price,
        current_mcap: accts.bonding_curve.current_mcap,
    });

    Ok(())
}

fn calculate_f44_cost(bonding_curve: &Account<BondingCurve>, token_amount: f64) -> Result<f64> {
    let first_price = bonding_curve.curve_slope * bonding_curve.token_reserves as f64
        + bonding_curve.initial_price;
    let last_price = bonding_curve.curve_slope
        * (bonding_curve.token_reserves as f64 - token_amount)
        + bonding_curve.initial_price;
    msg!(
        "The first price is {} and last price is {}",
        first_price.clone(),
        last_price.clone()
    );
    let average_price = (first_price + last_price) / 2.0;
    msg!("The average price of token is {}", average_price);

    let f44_amount = token_amount * average_price;
    msg!("The F44 amount is {}", f44_amount);

    Ok(f44_amount)
}
