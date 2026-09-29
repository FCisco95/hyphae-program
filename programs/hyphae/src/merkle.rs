use anchor_lang::prelude::Pubkey;
use solana_sha256_hasher::hashv;

// H-CONTRACT B8, byte for byte with packages/core/src/merkle.ts. Leaves and nodes hash under
// different prefixes, so an internal node can never be presented as a leaf.
const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;
pub const LEAF_LEN: usize = 89;

pub fn encode_leaf(
    wallet: &Pubkey,
    epoch_index: u64,
    score: u64,
    amount: u64,
    evidence_hash: &[u8; 32],
) -> [u8; LEAF_LEN] {
    let mut out = [0u8; LEAF_LEN];
    out[0] = LEAF_PREFIX;
    out[1..33].copy_from_slice(wallet.as_ref());
    out[33..41].copy_from_slice(&epoch_index.to_le_bytes());
    out[41..49].copy_from_slice(&score.to_le_bytes());
    out[49..57].copy_from_slice(&amount.to_le_bytes());
    out[57..89].copy_from_slice(evidence_hash);
    out
}

pub fn leaf_hash(
    wallet: &Pubkey,
    epoch_index: u64,
    score: u64,
    amount: u64,
    evidence_hash: &[u8; 32],
) -> [u8; 32] {
    hashv(&[&encode_leaf(
        wallet,
        epoch_index,
        score,
        amount,
        evidence_hash,
    )])
    .to_bytes()
}

// Pairs are sorted before hashing, so a proof carries no left/right flags.
pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    hashv(&[&[NODE_PREFIX], lo, hi]).to_bytes()
}

pub fn verify_proof(root: &[u8; 32], leaf: [u8; 32], proof: &[[u8; 32]]) -> bool {
    proof
        .iter()
        .fold(leaf, |acc, sibling| hash_pair(&acc, sibling))
        == *root
}
