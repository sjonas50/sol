use anchor_lang::prelude::*;

#[account]
pub struct Global {
    pub initialized: bool,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub owner_wallet: Pubkey,
    pub f44_mint: Pubkey,
    pub f44_vault: u64,
    pub f44_supply: u64,
    pub fee_basis_points: u64,
    pub mcap_limit: u64,
    pub create_fee: u64,
}

#[account]
pub struct BondingCurve {
    pub initial_price: u64,
    pub curve_slope: u64,
    pub token_reserves: u64,
    pub token_total_supply: u64,
    pub token_mint: Pubkey,
    pub mcap_limit: u64,
    pub complete: bool,
}