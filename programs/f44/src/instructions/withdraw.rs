use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token,TokenAccount,Transfer, transfer};

use crate::{
    state::{Global, BondingCurve},
    constants::{GLOBAL_STATE_SEED, BONDING_CURVE, SOL_VAULT_SEED},
    error::*,
};
use solana_program::{program::invoke_signed, system_instruction};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Account<'info, Global>,
    
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [SOL_VAULT_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [BONDING_CURVE, mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = owner_wallet
    )]
    pub associated_user: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub owner_wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.bonding_curve.complete == true, F44Code::BondingCurveNotComplete);
    require!(accts.global.owner_wallet == accts.owner_wallet.key(), F44Code::NotAuthorized);
    // withdraw all SOL and rest tokens to the owner (temporary)

    let binding = accts.mint.key();

    let (_, bump) = Pubkey::find_program_address(&[SOL_VAULT_SEED, binding.as_ref()], ctx.program_id);
    let vault_seeds = &[SOL_VAULT_SEED, binding.as_ref(), &[bump]];
    let signer = &[&vault_seeds[..]];

    
    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.owner_wallet.key(), accts.bonding_curve.real_sol_reserves),
        &[
            accts.vault.to_account_info().clone(),
            accts.owner_wallet.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        signer,
    )?;

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
        accts.bonding_curve.real_token_reserves,
    )?;

    Ok(())
}