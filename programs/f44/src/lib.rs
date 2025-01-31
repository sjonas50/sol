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

declare_id!("22ugTc9voq6eojXTNjV3i3QsD97h487pAyyJ7TsK3MVJ");

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
        agent_amount: u64,
        fee_amount: u64,
        create_fee: u64,
    ) -> Result<()> {
        instructions::set_params(
            ctx,
            fee_recipient,
            owner_wallet,
            agent_amount,
            fee_amount,
            create_fee,
        )
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit(ctx, amount)
    }
    pub fn create(ctx: Context<Create>, initial_price: f64, curve_slope: f64, amount: u64) -> Result<()> {
        instructions::create(ctx,initial_price, curve_slope, amount)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64, max_f44_amount: u64) -> Result<()> {
        instructions::buy(ctx, amount, max_f44_amount)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, min_f44_output: u64) -> Result<()> {
        instructions::sell(ctx, amount, min_f44_output)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
