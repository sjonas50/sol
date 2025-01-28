use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint,Token,TokenAccount, Transfer, transfer}
};
use std::mem::size_of;
use crate::{
    constants::{GLOBAL_STATE_SEED, SOL_VAULT_SEED, BONDING_CURVE, VAULT_SEED},
    state::{Global, BondingCurve},
    error::*,
    events::*,
};
use solana_program::{program::invoke, system_instruction};

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = user,
        seeds = [BONDING_CURVE, mint.key().as_ref()],
        bump,
        space = 8 + size_of::<BondingCurve>()
    )]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [VAULT_SEED, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub associated_user_account: Box<Account<'info, TokenAccount>>,

    pub f44_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub associated_user_f44_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,
   
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn create(ctx: Context<Create>, initial_price: u64, curve_slope: u64, amount: u64) -> Result<()> {
    let accts = ctx.accounts;
    let decimals = accts.mint.decimals;
    let f44_decimals = accts.f44_mint.decimals;

    require!(accts.global.initialized == true, F44Code::NotInitialized);
    require!(amount == accts.global.agent_amount * 10_u64.pow(decimals), RoundError::NotEnoughAmount);

    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_user_account.to_account_info().clone(),
            to: accts.associated_bonding_curve.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;

    msg!(
        "transfer agent tokens from user to bonding curve - Mint: {}, amount: {}, User: {}, Bonding Curve: {}",
        accts.mint.key(),
        amount,
        accts.user.key(),
        accts.bonding_curve.key()
    );
    
    let cpi_fee_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_user_f44_account.to_account_info().clone(),
            to: accts.f44_vault.to_account_info().clone(),
            authority: accts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_fee_ctx, accts.global.fee_amount)?;
    msg!(
        "transfer f44 tokens from user to bonding reserve pool as fee - F44: {}, amount: {}",
        accts.f44_mint.key(),
        accts.global.fee_amount / 10_u64.pow(f44_decimals),
    );
    // init the bonding curve
    accts.bonding_curve.initial_price = initial_price;
    accts.bonding_curve.curve_slope = curve_slope;
    accts.bonding_curve.token_reserves = 0;
    accts.bonding_curve.token_total_supply = amount;
    accts.bonding_curve.mcap_limit = 100000;
    accts.bonding_curve.complete = false;
    accts.bonding_curve.token_mint = accts.mint.key();

    // Log the event details
    msg!(
        "CreateEvent - Mint: {}, BondingCurve: {}, User: {}",
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.user.key()
    );
    
    emit!{
        CreateEvent {
            mint: accts.mint.key(),
            bonding_curve: accts.bonding_curve.key(),
            user: accts.user.key()
        }
    }

    Ok(())
}