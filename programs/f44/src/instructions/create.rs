use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token,TokenAccount, Transfer, transfer};
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

    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,
   
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create(ctx: Context<Create>, amount: u64) -> Result<()> {
    let global: &Box<Account<Global>> = &ctx.accounts.global;
    let bonding_curve: &mut Box<Account<BondingCurve>> = &mut ctx.accounts.bonding_curve;

    require!(global.initialized == true, F44Code::NotInitialized);

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.associated_user_account.to_account_info().clone(),
            to: ctx.accounts.associated_bonding_curve.to_account_info().clone(),
            authority: ctx.accounts.user.to_account_info().clone(),
        },
    );
    transfer(cpi_ctx, amount)?;
    
    invoke(
        &system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            global.create_fee
        ),
        &[
            ctx.accounts.user.to_account_info().clone(),
            ctx.accounts.vault.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
        ],
    )?;
    // init the bonding curve
    bonding_curve.virtual_token_reserves = global.initial_virtual_token_reserves;
    bonding_curve.virtual_sol_reserves = global.initial_virtual_sol_reserves;
    bonding_curve.real_token_reserves = amount;
    bonding_curve.real_sol_reserves = 0;
    bonding_curve.token_total_supply = global.token_total_supply;
    bonding_curve.mcap_limit = global.mcap_limit;
    bonding_curve.complete = false;
    bonding_curve.token_mint = ctx.accounts.mint.key();

    // Log the event details
    msg!(
        "CreateEvent - Mint: {}, BondingCurve: {}, User: {}",
        ctx.accounts.mint.key(),
        bonding_curve.key(),
        ctx.accounts.user.key()
    );
    
    emit!{
        CreateEvent {
            mint: ctx.accounts.mint.key(),
            bonding_curve: bonding_curve.key(),
            user: ctx.accounts.user.key()
        }
    }

    Ok(())
}