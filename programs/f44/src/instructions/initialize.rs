use anchor_lang::prelude::*;

use crate::{
    constants::GLOBAL_STATE_SEED,
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
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let global = &mut ctx.accounts.global;

    require!(global.initialized == false, F44Code::AlreadyInitialized);
    
    global.authority = ctx.accounts.owner.key();
    global.initialized = true;
    
    Ok(())
}