use anchor_lang::prelude::*;
use anchor_lang::solana_program::msg;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, transfer, Mint, Token, TokenAccount, Transfer}
};
use std::mem::size_of;
use crate::{
    constants::{GLOBAL_STATE_SEED, F44_VAULT_SEED, BONDING_CURVE, VAULT_SEED},
    state::{Global, BondingCurve},
    error::*,
    events::*,
};
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
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

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut, 
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub f44_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub associated_owner_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create(ctx: Context<Create>, initial_price: f64, curve_slope: f64, amount: u64) -> Result<()> {
    let accts = ctx.accounts;
    let decimals = accts.mint.decimals;
    let f44_decimals = accts.f44_mint.decimals;

    require!(accts.global.initialized == true, F44Code::NotInitialized);
    require!(amount == accts.global.agent_amount * 10_u64.pow(decimals.into()), F44Code::NotEnoughAmount);

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
    transfer(cpi_fee_ctx, accts.global.fee_amount * 10_u64.pow(f44_decimals.into()))?;
    msg!(
        "transfer f44 tokens from user to bonding reserve pool as fee - F44: {}, amount: {}",
        accts.f44_mint.key(),
        accts.global.fee_amount * 10_u64.pow(f44_decimals.into()),
    );
    // init the bonding curve
    accts.bonding_curve.initial_price = initial_price;
    accts.bonding_curve.current_price = initial_price;
    accts.bonding_curve.current_mcap = 0.0;
    accts.bonding_curve.curve_slope = curve_slope;
    accts.bonding_curve.token_reserves = 0.0;
    accts.bonding_curve.token_total_supply = (amount / 10_u64.pow(decimals.into())) as f64;
    accts.bonding_curve.mcap_limit = 100000.0;
    accts.bonding_curve.complete = false;
    accts.bonding_curve.token_mint = accts.mint.key();

    // Log the event details
    msg!(
        "CreateEvent - Mint: {}, bondingCurve: {}, user: {}, initialPrice: {}",
        accts.mint.key(),
        accts.bonding_curve.key(),
        accts.user.key(),
        accts.bonding_curve.initial_price,
    );
    
    emit!{
        CreateEvent {
            mint: accts.mint.key(),
            bonding_curve: accts.bonding_curve.key(),
            user: accts.user.key(),
            initial_price: accts.bonding_curve.initial_price,
        }
    }

    Ok(())
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;
    // Deposit f44 to the vault account
    let cpi_ctx = CpiContext::new(
        accts.token_program.to_account_info(),
        Transfer {
            from: accts.associated_owner_account.to_account_info().clone(),
            to: accts.f44_vault.to_account_info().clone(),
            authority: accts.owner.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;

    accts.global.f44_supply += amount;

    Ok(())
}