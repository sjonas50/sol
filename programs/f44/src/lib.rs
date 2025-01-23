pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod events;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;
pub use state::*;
pub use events::*;

declare_id!("C67YgrXJvyN9AY8jE6yLccNDNfNxg9LKS7LTJDKsMqnv");

#[program]
pub mod f44 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn set_params(
        ctx: Context<SetParams>,
        fee_recipient: Pubkey,
        owner_wallet: Pubkey,
        initial_virtual_token_reserves: u64,
        initial_virtual_sol_reserves: u64,
        initial_real_token_reserves: u64,
        token_total_supply: u64,
        mcap_limit: u64,
        fee_basis_points: u64,
        create_fee: u64,
    ) -> Result<()> {
        instructions::set_params(
            ctx,
            fee_recipient,
            owner_wallet,
            initial_virtual_token_reserves,
            initial_virtual_sol_reserves,
            initial_real_token_reserves,
            token_total_supply,
            mcap_limit,
            fee_basis_points,
            create_fee,
        )
    }

    pub fn create(ctx: Context<Create>, amount: u64) -> Result<()> {
        instructions::create(ctx, amount)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64, max_sol_cost: u64) -> Result<()> {
        instructions::buy(ctx, amount, max_sol_cost)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, min_sol_output: u64) -> Result<()> {
        instructions::sell(ctx, amount, min_sol_output)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
    
}
