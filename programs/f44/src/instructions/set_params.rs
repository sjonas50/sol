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

pub fn set_params(ctx: Context<SetParams>, fee_recipient: Pubkey,owner_wallet: Pubkey, mcap_limit: u64, fee_basis_points: u64, create_fee: u64) -> Result<()> {
    let global = &mut ctx.accounts.global;

    require!(global.authority == ctx.accounts.user.key(), F44Code::NotAuthorized);

    global.fee_recipient = fee_recipient;
    global.owner_wallet = owner_wallet;
    global.mcap_limit = mcap_limit;
    global.fee_basis_points = fee_basis_points;
    global.create_fee = create_fee;

    msg!("Set params\n
        fee recipient is {:?}\n
        owner wallet is {:?}\n
        mcap limit is {:?}\n
        fee basis points is {:?}\n
        create fee is {:?}\n
        authority {:?}", 
        global.fee_recipient,
        global.owner_wallet,
        global.mcap_limit,
        global.fee_basis_points,
        global.create_fee,
    );

    Ok(())
}