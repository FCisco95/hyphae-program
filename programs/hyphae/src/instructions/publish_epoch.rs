use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::HyphaeError;
use crate::state::{Community, Epoch, Vault};

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct PublishEpoch<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    // The admin is part of the seeds, so only the key that created the community can publish.
    #[account(
        mut,
        seeds = [COMMUNITY_SEED, community.mint.as_ref(), admin.key().as_ref()],
        bump = community.bump
    )]
    pub community: Account<'info, Community>,
    #[account(
        mut,
        seeds = [VAULT_SEED, community.key().as_ref()],
        bump = community.vault_bump
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: only receives the fee; it must be the recipient fixed at initialization (P7).
    #[account(mut, address = community.fee_recipient)]
    pub fee_recipient: UncheckedAccount<'info>,
    // One publish per epoch index: a second init of the same address fails.
    #[account(
        init,
        payer = admin,
        space = 8 + Epoch::INIT_SPACE,
        seeds = [EPOCH_SEED, community.key().as_ref(), &index.to_le_bytes()],
        bump
    )]
    pub epoch: Account<'info, Epoch>,
    pub system_program: Program<'info, System>,
}

pub fn process_publish_epoch(
    ctx: Context<PublishEpoch>,
    index: u64,
    root: [u8; 32],
    audit_hash: [u8; 32],
    gross_lamports: u64,
    allocated_lamports: u64,
) -> Result<()> {
    require!(root != [0u8; 32], HyphaeError::EmptyRoot);
    require!(gross_lamports > 0, HyphaeError::ZeroGross);
    // P7: floor(G × 300 / 10,000); rounding favours contributors.
    let fee_lamports = (u128::from(gross_lamports) * u128::from(FEE_BPS) / u128::from(BPS)) as u64;
    let net_lamports = gross_lamports - fee_lamports;
    // A positive allocation also means nobody-payable can never publish or take a fee (PG6).
    require!(
        allocated_lamports > 0 && allocated_lamports <= net_lamports,
        HyphaeError::InvalidAllocation
    );

    // P6: the pot comes out of the unassigned balance: never the vault's own rent, never money
    // an earlier epoch allocated and nobody has claimed yet.
    let vault = ctx.accounts.vault.to_account_info();
    let rent = Rent::get()?.minimum_balance(vault.data_len());
    let unassigned = vault
        .lamports()
        .checked_sub(rent)
        .and_then(|free| free.checked_sub(ctx.accounts.community.outstanding_lamports))
        .ok_or(HyphaeError::InsufficientVaultBalance)?;
    require!(
        gross_lamports <= unassigned,
        HyphaeError::InsufficientVaultBalance
    );

    if fee_lamports > 0 {
        vault.sub_lamports(fee_lamports)?;
        ctx.accounts.fee_recipient.add_lamports(fee_lamports)?;
    }
    // Only the allocated total is reserved: cap remainder and dust stay unassigned (P11).
    let community = &mut ctx.accounts.community;
    community.outstanding_lamports = community
        .outstanding_lamports
        .checked_add(allocated_lamports)
        .ok_or(HyphaeError::MathOverflow)?;

    ctx.accounts.epoch.set_inner(Epoch {
        community: community.key(),
        index,
        root,
        audit_hash,
        gross_lamports,
        fee_lamports,
        allocated_lamports,
        claimed_lamports: 0,
        published_at: Clock::get()?.unix_timestamp,
        bump: ctx.bumps.epoch,
    });
    Ok(())
}
