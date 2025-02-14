use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    constants::{BONDING_CURVE, F44_VAULT_SEED, GLOBAL_STATE_SEED},
    error::*,
    events::*,
    state::{BondingCurve, Global},
};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>, // the mint address of token

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

    #[account(mut)]
    pub f44_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
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
// amount is the agent token amount
// max_f44_amount is the max f44 token amount that will be used as payment
pub fn buy(ctx: Context<Buy>, amount: u64, max_f44_amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    // Basic parameter validation
    require!(amount > 0, F44Code::ZeroAmount);
    require!(max_f44_amount > 0, F44Code::ZeroAmount);
    require!(!accts.bonding_curve.complete, F44Code::BondingCurveComplete);

    let bonding_curve = &accts.bonding_curve;
    let decimals = accts.mint.decimals;
    let f44_decimals = accts.f44_mint.decimals;

    // Calculate F44 cost with input validation
    let token_amount = amount as f64 / 10_u64.pow(decimals.into()) as f64;
    let f44_amount = calculate_f44_cost(bonding_curve, token_amount)?;
    accts.bonding_curve.current_price = bonding_curve.curve_slope
        * (bonding_curve.token_reserves + token_amount)
        + bonding_curve.initial_price;

    // Convert to token units with overflow protection
    let f44_transfer_amount = (f44_amount * 10_u64.pow(f44_decimals.into()) as f64) as u64;

    // Validate transfer amount
    require!(f44_transfer_amount > 0, F44Code::InvalidAmount);
    require!(
        f44_transfer_amount <= max_f44_amount,
        F44Code::TooMuchF44Required
    );

    // Check bonding curve has sufficient tokens
    require!(
        accts.associated_bonding_curve.amount >= amount,
        F44Code::InsufficientLiquidity
    );

    // send f44 token to the f44 reserve pool
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_user_f44_account.to_account_info().clone(),
            to: accts.f44_vault.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, f44_transfer_amount)?;

    // Update F44 supply with overflow check
    accts.global.f44_supply = accts
        .global
        .f44_supply
        .checked_add(f44_transfer_amount)
        .ok_or(F44Code::MathOverflow)?;

    // send token from agent token vault account to user
    let binding = accts.mint.key();

    let (_, bump) =
        Pubkey::find_program_address(&[BONDING_CURVE, binding.as_ref()], ctx.program_id);
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
    transfer(cpi_ctx.with_signer(signer), amount)?;

    // Update bonding curve state
    let token_amount = amount as f64 / 10_u64.pow(decimals.into()) as f64;
    accts.bonding_curve.token_reserves += token_amount;

    // Calculate market cap
    let macp = (accts.bonding_curve.curve_slope * accts.bonding_curve.token_reserves
        + accts.bonding_curve.initial_price)
        * accts.bonding_curve.token_total_supply;
    accts.bonding_curve.current_mcap = macp;

    msg!("Current market cap is {}", macp);

    if macp > accts.bonding_curve.mcap_limit {
        accts.bonding_curve.complete = true;

        msg!(
            "CompleteEvent - user: {}, mint: {}, bondingCurve: {}, timestamp: {}",
            accts.user.key(),
            accts.mint.key(),
            accts.bonding_curve.key(),
            accts.clock.unix_timestamp
        );

        emit!(CompleteEvent {
            user: accts.user.key(),
            mint: accts.mint.key(),
            bonding_curve: accts.bonding_curve.key(),
            timestamp: accts.clock.unix_timestamp,
        });
    }

    msg!(
        "TradeEvent - type: Buy, user: {}, mint: {}, bondingCurve: {}, timestamp: {}, f44Amount: {}, tokenAmount: {}, isBuy: {}, tokenReserves: {}, currentPrice: {}, currentMcap: {}",
        accts.user.key(),
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.clock.unix_timestamp,
        f44_amount,
        amount,
        true,
        accts.bonding_curve.token_reserves,
        accts.bonding_curve.current_price,
        accts.bonding_curve.current_mcap,
    );

    emit!(TradeEvent {
        mint: accts.mint.key(),
        amount: f44_transfer_amount,
        token_amount: amount,
        is_buy: true,
        user: accts.user.key(),
        timestamp: accts.clock.unix_timestamp,
        token_reserves: accts.bonding_curve.token_reserves,
        last_price: accts.bonding_curve.current_price,
        current_mcap: accts.bonding_curve.current_mcap,
    });

    Ok(())
}

fn calculate_f44_cost(bonding_curve: &Account<BondingCurve>, token_amount: f64) -> Result<f64> {
    // Input validation
    require!(token_amount >= 0.0, F44Code::InvalidAmount);
    require!(token_amount.is_finite(), F44Code::InvalidAmount);
    require!(bonding_curve.curve_slope >= 0.0, F44Code::InvalidSlope);
    require!(bonding_curve.curve_slope.is_finite(), F44Code::InvalidSlope);
    require!(bonding_curve.initial_price >= 0.0, F44Code::InvalidPrice);
    require!(
        bonding_curve.initial_price.is_finite(),
        F44Code::InvalidPrice
    );
    require!(
        bonding_curve.token_reserves >= 0.0,
        F44Code::InvalidReserves
    );
    require!(
        bonding_curve.token_reserves.is_finite(),
        F44Code::InvalidReserves
    );

    // Price calculations
    let first_price =
        bonding_curve.curve_slope * bonding_curve.token_reserves + bonding_curve.initial_price;
    let last_price = bonding_curve.curve_slope * (bonding_curve.token_reserves + token_amount)
        + bonding_curve.initial_price;

    // Validate calculated prices
    require!(first_price >= 0.0, F44Code::InvalidCalculation);
    require!(first_price.is_finite(), F44Code::InvalidCalculation);
    require!(last_price >= 0.0, F44Code::InvalidCalculation);
    require!(last_price.is_finite(), F44Code::InvalidCalculation);

    msg!(
        "The first price is {} and last price is {}",
        first_price,
        last_price
    );

    let average_price = (first_price + last_price) / 2.0;
    msg!("The average price of token is {}", average_price);

    let f44_amount = token_amount * average_price;
    msg!("The F44 amount is {}", f44_amount);

    // Validate final result
    require!(f44_amount >= 0.0, F44Code::InvalidCalculation);
    require!(f44_amount.is_finite(), F44Code::InvalidCalculation);
    require!(f44_amount <= f64::MAX / 2.0, F44Code::InvalidCalculation); // Prevent potential overflow

    Ok(f44_amount)
}
