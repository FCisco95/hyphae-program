use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::HyphaeError;
use crate::state::{Community, Vault};

// SPL Token mints are 82 bytes, with is_initialized at byte 45. Token-2022 mints share that base
// layout; with extensions it is padded to 165 bytes and byte 165 is the account type (1 = mint).
const MINT_LEN: usize = 82;
const MINT_INITIALIZED_OFFSET: usize = 45;
const ACCOUNT_TYPE_OFFSET: usize = 165;
const ACCOUNT_TYPE_MINT: u8 = 1;

fn is_mint(info: &AccountInfo) -> bool {
    let token_2022 = *info.owner == TOKEN_2022_PROGRAM_ID;
    if !token_2022 && *info.owner != TOKEN_PROGRAM_ID {
        return false;
    }
    let Ok(data) = info.try_borrow_data() else {
        return false;
    };
    let shape = data.len() == MINT_LEN
        || (token_2022
            && data.len() > ACCOUNT_TYPE_OFFSET
            && data[ACCOUNT_TYPE_OFFSET] == ACCOUNT_TYPE_MINT);
    shape && data[MINT_INITIALIZED_OFFSET] == 1
}

#[derive(Accounts)]
pub struct InitializeCommunity<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: only its address is used, as a seed; the constraint checks it is a mint, so a
    /// wallet or token account address cannot be registered by mistake.
    #[account(constraint = is_mint(&mint) @ HyphaeError::NotAMint)]
    pub mint: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Community::INIT_SPACE,
        seeds = [COMMUNITY_SEED, mint.key().as_ref(), admin.key().as_ref()],
        bump
    )]
    pub community: Account<'info, Community>,
    #[account(
        init,
        payer = admin,
        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_SEED, community.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

pub fn process_initialize_community(
    ctx: Context<InitializeCommunity>,
    fee_recipient: Pubkey,
) -> Result<()> {
    require!(
        fee_recipient != Pubkey::default()
            && fee_recipient != ctx.accounts.community.key()
            && fee_recipient != ctx.accounts.vault.key(),
        HyphaeError::InvalidFeeRecipient
    );
    ctx.accounts.community.set_inner(Community {
        mint: ctx.accounts.mint.key(),
        admin: ctx.accounts.admin.key(),
        fee_recipient,
        outstanding_lamports: 0,
        bump: ctx.bumps.community,
        vault_bump: ctx.bumps.vault,
    });
    ctx.accounts.vault.set_inner(Vault {
        bump: ctx.bumps.vault,
    });
    Ok(())
}
