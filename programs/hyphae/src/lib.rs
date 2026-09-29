pub mod constants;
pub mod error;
pub mod instructions;
pub mod merkle;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E");

// A per-community SOL vault, epoch roots and one-time claims (H-CONTRACT Part B, payment
// rulings P1–P16). No instruction withdraws or sweeps the vault (P4).
#[program]
pub mod hyphae {
    use super::*;

    pub fn initialize_community(
        ctx: Context<InitializeCommunity>,
        fee_recipient: Pubkey,
    ) -> Result<()> {
        process_initialize_community(ctx, fee_recipient)
    }

    pub fn publish_epoch(
        ctx: Context<PublishEpoch>,
        index: u64,
        root: [u8; 32],
        audit_hash: [u8; 32],
        gross_lamports: u64,
        allocated_lamports: u64,
    ) -> Result<()> {
        process_publish_epoch(
            ctx,
            index,
            root,
            audit_hash,
            gross_lamports,
            allocated_lamports,
        )
    }

    pub fn claim(
        ctx: Context<Claim>,
        score: u64,
        amount: u64,
        evidence_hash: [u8; 32],
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        process_claim(ctx, score, amount, evidence_hash, proof)
    }
}
