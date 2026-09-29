use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::HyphaeError;
use crate::merkle::{leaf_hash, verify_proof};
use crate::state::{ClaimReceipt, Community, Epoch, Vault};

#[derive(Accounts)]
pub struct Claim<'info> {
    // The leaf's wallet: the leaf is recomputed with this key, so nobody can claim for another.
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(
        mut,
        seeds = [COMMUNITY_SEED, community.mint.as_ref(), community.admin.as_ref()],
        bump = community.bump
    )]
    pub community: Account<'info, Community>,
    #[account(
        mut,
        seeds = [VAULT_SEED, community.key().as_ref()],
        bump = community.vault_bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        has_one = community,
        seeds = [EPOCH_SEED, community.key().as_ref(), &epoch.index.to_le_bytes()],
        bump = epoch.bump
    )]
    pub epoch: Account<'info, Epoch>,
    // P13: the receipt pays a leaf once; a second init of the same address fails.
    #[account(
        init,
        payer = claimant,
        space = 8 + ClaimReceipt::INIT_SPACE,
        seeds = [CLAIM_SEED, epoch.key().as_ref(), claimant.key().as_ref()],
        bump
    )]
    pub receipt: Account<'info, ClaimReceipt>,
    pub system_program: Program<'info, System>,
}

pub fn process_claim(
    ctx: Context<Claim>,
    score: u64,
    amount: u64,
    evidence_hash: [u8; 32],
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    require!(amount > 0, HyphaeError::ZeroAmount);
    let claimant = ctx.accounts.claimant.key();
    let epoch = &mut ctx.accounts.epoch;
    let leaf = leaf_hash(&claimant, epoch.index, score, amount, &evidence_hash);
    require!(
        verify_proof(&epoch.root, leaf, &proof),
        HyphaeError::InvalidProof
    );

    // Whatever the root holds, an epoch never pays out more than it reserved at publish.
    let claimed = epoch
        .claimed_lamports
        .checked_add(amount)
        .ok_or(HyphaeError::MathOverflow)?;
    require!(
        claimed <= epoch.allocated_lamports,
        HyphaeError::EpochOverclaimed
    );
    epoch.claimed_lamports = claimed;
    let community = &mut ctx.accounts.community;
    community.outstanding_lamports = community
        .outstanding_lamports
        .checked_sub(amount)
        .ok_or(HyphaeError::MathOverflow)?;

    ctx.accounts.vault.sub_lamports(amount)?;
    ctx.accounts.claimant.add_lamports(amount)?;

    ctx.accounts.receipt.set_inner(ClaimReceipt {
        epoch: epoch.key(),
        wallet: claimant,
        score,
        amount,
        evidence_hash,
        claimed_at: Clock::get()?.unix_timestamp,
        bump: ctx.bumps.receipt,
    });
    Ok(())
}
