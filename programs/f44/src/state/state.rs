use anchor_lang::prelude::*;

#[account]
pub struct Global {
    pub initialized: bool,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub owner_wallet: Pubkey,
    pub f44_mint: Pubkey,
    pub f44_vault: Pubkey,
    pub f44_supply: u64,
    pub fee_amount: u64,
    pub agent_amount: u64,
    pub create_fee: u64,
}

#[account]
pub struct BondingCurve {
    pub initial_price: f64,
    pub curve_slope: f64,
    pub token_reserves: f64,
    pub token_total_supply: f64,
    pub token_mint: Pubkey,
    pub mcap_limit: f64,
    pub complete: bool,
}