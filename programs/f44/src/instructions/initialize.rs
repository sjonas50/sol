use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,Token,TokenAccount};

use crate::{
    constants::{GLOBAL_STATE_SEED, F44_VAULT_SEED},
    state::Global,
    error::*,
};
use std::mem::size_of;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = owner, 
        seeds = [GLOBAL_STATE_SEED],
        bump,
        space = 8 + size_of::<Global>()
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub f44_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = owner,
        seeds = [F44_VAULT_SEED, f44_mint.key().as_ref()],
        bump,
        token::mint = f44_mint,
        token::authority = global,
    )]
    pub f44_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.global.initialized == false, F44Code::AlreadyInitialized);
    
    accts.global.authority = accts.owner.key();
    accts.global.initialized = true;
    accts.global.f44_mint = accts.f44_mint.key();
    accts.global.f44_vault = accts.f44_vault.key();
    
    Ok(())
}


