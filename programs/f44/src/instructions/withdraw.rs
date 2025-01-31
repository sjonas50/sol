use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint,Token,TokenAccount,Transfer, transfer}
};

use crate::{
    state::{Global, BondingCurve},
    constants::{GLOBAL_STATE_SEED, BONDING_CURVE, F44_VAULT_SEED},
    error::*,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Account<'info, Global>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,

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
        init_if_needed,
        payer = owner_wallet,
        associated_token::mint = mint,
        associated_token::authority = owner_wallet
    )]
    pub associated_user: Account<'info, TokenAccount>,

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
        payer = owner_wallet,
        associated_token::mint = f44_mint,
        associated_token::authority = owner_wallet
    )]
    pub associated_user_f44_account: Box<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub owner_wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.bonding_curve.complete == true, F44Code::BondingCurveNotComplete);
    require!(accts.global.owner_wallet == accts.owner_wallet.key(), F44Code::NotAuthorized);
    // withdraw all SOL and rest tokens to the owner (temporary)

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
        (accts.bonding_curve.token_total_supply - accts.bonding_curve.token_reserves) as u64,
    )?;

    let (_, bump) =  Pubkey::find_program_address(&[GLOBAL_STATE_SEED], ctx.program_id);
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
        accts.global.f44_supply / 100,
    )?;

    accts.global.f44_supply -= accts.global.f44_supply / 100;

    Ok(())
}