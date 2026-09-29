use anchor_lang::prelude::*;

pub const COMMUNITY_SEED: &[u8] = b"community";
pub const VAULT_SEED: &[u8] = b"vault";
pub const EPOCH_SEED: &[u8] = b"epoch";
pub const CLAIM_SEED: &[u8] = b"claim";

// P7: the Hyphae fee is 3% of an epoch's gross pot, floored.
#[constant]
pub const FEE_BPS: u64 = 300;
pub const BPS: u64 = 10_000;

pub const TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
