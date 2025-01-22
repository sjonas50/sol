use anchor_lang::prelude::*;
use crate::{
    state::Global,
    constants::GLOBAL_STATE_SEED,
    error::*,
};

#[derive(Accounts)]
pub struct SetParams<'info> {
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn set_params(ctx: Context<SetParams>, fee_recipient: Pubkey,owner_wallet: Pubkey,  initial_virtual_token_reserves: u64, initial_virtual_sol_reserves: u64, initial_real_token_reserves: u64, token_total_supply: u64, mcap_limit: u64, fee_basis_points: u64, create_fee: u64) -> Result<()> {
    let global = &mut ctx.accounts.global;

    require!(global.authority == ctx.accounts.user.key(), F44Code::NotAuthorized);

    global.fee_recipient = fee_recipient;
    global.owner_wallet = owner_wallet;
    global.initial_virtual_token_reserves = initial_virtual_token_reserves;
    global.initial_virtual_sol_reserves = initial_virtual_sol_reserves;
    global.initial_real_token_reserves = initial_real_token_reserves;
    global.token_total_supply = token_total_supply;
    global.mcap_limit = mcap_limit;
    global.fee_basis_points = fee_basis_points;
    global.create_fee = create_fee;
    global.authority = ctx.accounts.user.key();

    Ok(())
}