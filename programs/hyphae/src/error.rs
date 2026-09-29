use anchor_lang::prelude::*;

#[error_code]
pub enum HyphaeError {
    #[msg("The mint is not an initialized SPL Token or Token-2022 mint")]
    NotAMint,
    #[msg("The fee recipient must be a real address other than the community and its vault")]
    InvalidFeeRecipient,
    #[msg("An epoch root cannot be all zeros")]
    EmptyRoot,
    #[msg("The gross pot must be positive")]
    ZeroGross,
    #[msg("The allocated total must be positive and at most the net pot")]
    InvalidAllocation,
    #[msg("The gross pot exceeds the vault's unassigned balance")]
    InsufficientVaultBalance,
    #[msg("A claim must pay a positive amount")]
    ZeroAmount,
    #[msg("The leaf and proof do not match the epoch root")]
    InvalidProof,
    #[msg("The claim would exceed the epoch's allocated total")]
    EpochOverclaimed,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
