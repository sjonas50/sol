use anchor_lang::prelude::*;

#[error_code]
pub enum F44Code {
    #[msg("The given account is not authorized to execute this instruction.")]
    NotAuthorized,

    #[msg("The given account is not valid fee recipient.")]
    UnValidFeeRecipient,

    #[msg("The program is already initialized.")]
    AlreadyInitialized,

    #[msg("slippage: Too much F44 Token required to buy the given amount of tokens.")]
    TooMuchF44Required,

    #[msg("slippage: Too little SOL received to sell the given amount of tokens.")]
    TooLittleSolReceived,

    #[msg("The mint does not match the bonding curve.")]
    MintDoesNotMatchBondingCurve,

    #[msg("The agent token amount is not enough to create the bonding curve.")]
    NotEnoughAmount,

    #[msg("The bonding curve has completed and liquidity migrated to raydium.")]
    BondingCurveComplete,

    #[msg("The bonding curve has not completed.")]
    BondingCurveNotComplete,

    #[msg("The program is not initialized.")]
    NotInitialized,
    
    #[msg("Math operation overflow.")]
    MathOverflow,

    #[msg("Amount should be bigger than 0.")]
    ZeroAmount
}