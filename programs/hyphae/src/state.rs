use anchor_lang::prelude::*;

// One community per (mint, admin): the vault address names whose publish key it trusts.
#[account]
#[derive(InitSpace)]
pub struct Community {
    pub mint: Pubkey,
    // P5: the only key that can publish an epoch.
    pub admin: Pubkey,
    // P7/P8: fixed at initialization; no instruction changes it.
    pub fee_recipient: Pubkey,
    // Allocated and not yet claimed, across every epoch. Publish may only assign the rest.
    pub outstanding_lamports: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

// Holds the community's SOL (P3, P4). Anyone can transfer SOL in; only publish (the fee) and
// claim move it out.
#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Epoch {
    pub community: Pubkey,
    // The Hyphae epoch index, also committed in every leaf.
    pub index: u64,
    pub root: [u8; 32],
    // H-CONTRACT B9: the epoch audit manifest hash, which the root alone cannot stand in for.
    pub audit_hash: [u8; 32],
    pub gross_lamports: u64,
    pub fee_lamports: u64,
    pub allocated_lamports: u64,
    pub claimed_lamports: u64,
    pub published_at: i64,
    pub bump: u8,
}

// P13: a claim receipt is the evidence of payment; its existence stops a second claim.
#[account]
#[derive(InitSpace)]
pub struct ClaimReceipt {
    pub epoch: Pubkey,
    pub wallet: Pubkey,
    pub score: u64,
    pub amount: u64,
    pub evidence_hash: [u8; 32],
    pub claimed_at: i64,
    pub bump: u8,
}
