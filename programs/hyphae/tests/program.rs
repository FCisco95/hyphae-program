mod common;

use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use common::{dec, hex32, proof, tree, vectors};
use hyphae::state::{ClaimReceipt, Community, Epoch};
use litesvm::LiteSVM;
use serde_json::Value;
use solana_account::Account;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

const SOL: u64 = 1_000_000_000;
const TOKEN_PROGRAM: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM: Pubkey =
    Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const TX_FEE: u64 = 5_000;

// Anchor error codes: framework constraints and this program's errors (6000 + variant index).
const ACCOUNT_NOT_INITIALIZED: u32 = 3012;
const CONSTRAINT_SEEDS: u32 = 2006;
const CONSTRAINT_ADDRESS: u32 = 2012;
const NOT_A_MINT: u32 = 6000;
const INVALID_FEE_RECIPIENT: u32 = 6001;
const EMPTY_ROOT: u32 = 6002;
const ZERO_GROSS: u32 = 6003;
const INVALID_ALLOCATION: u32 = 6004;
const INSUFFICIENT_VAULT_BALANCE: u32 = 6005;
const ZERO_AMOUNT: u32 = 6006;
const INVALID_PROOF: u32 = 6007;
const EPOCH_OVERCLAIMED: u32 = 6008;

type Outcome = Result<(), TransactionError>;

fn custom(outcome: &Outcome) -> Option<u32> {
    match outcome {
        Err(TransactionError::InstructionError(_, InstructionError::Custom(code))) => Some(*code),
        _ => None,
    }
}

struct Env {
    svm: LiteSVM,
    admin: Keypair,
    mint: Pubkey,
    fee_recipient: Pubkey,
    community: Pubkey,
    vault: Pubkey,
}

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &hyphae::ID).0
}
fn community_pda(mint: &Pubkey, admin: &Pubkey) -> Pubkey {
    pda(&[b"community", mint.as_ref(), admin.as_ref()])
}
fn vault_pda(community: &Pubkey) -> Pubkey {
    pda(&[b"vault", community.as_ref()])
}
fn epoch_pda(community: &Pubkey, index: u64) -> Pubkey {
    pda(&[b"epoch", community.as_ref(), &index.to_le_bytes()])
}
fn receipt_pda(epoch: &Pubkey, wallet: &Pubkey) -> Pubkey {
    pda(&[b"claim", epoch.as_ref(), wallet.as_ref()])
}

fn send(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> Outcome {
    // A fresh blockhash per transaction, so a repeated instruction is a new transaction and
    // reaches the program instead of failing as an already processed signature.
    svm.expire_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&signers[0].pubkey()),
        signers,
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).map(|_| ()).map_err(|f| f.err)
}

fn funded(svm: &mut LiteSVM, lamports: u64) -> Keypair {
    let k = Keypair::new();
    svm.airdrop(&k.pubkey(), lamports).unwrap();
    k
}

fn owned_account(svm: &mut LiteSVM, owner: Pubkey, data: Vec<u8>) -> Pubkey {
    let key = Pubkey::new_unique();
    svm.set_account(
        key,
        Account {
            lamports: SOL,
            data,
            owner,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    key
}

// The SPL mint layout: authority (36 bytes), supply (8), decimals (1), is_initialized (1), ...
const MINT_INITIALIZED: usize = 45;

fn mint_account(svm: &mut LiteSVM, owner: Pubkey) -> Pubkey {
    let mut data = vec![0; 82];
    data[MINT_INITIALIZED] = 1;
    owned_account(svm, owner, data)
}

// Token-2022 accounts with extensions: base layout padded to 165 bytes, then the account type
// (1 = mint, 2 = token account), then the extensions.
fn token_2022_with_extensions(svm: &mut LiteSVM, account_type: u8) -> Pubkey {
    let mut data = vec![0; 170];
    data[MINT_INITIALIZED] = 1;
    data[165] = account_type;
    owned_account(svm, TOKEN_2022_PROGRAM, data)
}

fn initialize_ix(admin: &Pubkey, mint: &Pubkey, fee_recipient: Pubkey) -> Instruction {
    let community = community_pda(mint, admin);
    Instruction {
        program_id: hyphae::ID,
        accounts: hyphae::accounts::InitializeCommunity {
            admin: *admin,
            mint: *mint,
            community,
            vault: vault_pda(&community),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: hyphae::instruction::InitializeCommunity { fee_recipient }.data(),
    }
}

fn setup() -> Env {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        hyphae::ID,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/hyphae.so"),
    )
    .unwrap();
    let admin = funded(&mut svm, 10 * SOL);
    let mint = mint_account(&mut svm, TOKEN_PROGRAM);
    // An existing wallet, as the P8 fee address will be.
    let fee_recipient = funded(&mut svm, SOL).pubkey();
    send(
        &mut svm,
        initialize_ix(&admin.pubkey(), &mint, fee_recipient),
        &[&admin],
    )
    .unwrap();
    let community = community_pda(&mint, &admin.pubkey());
    Env {
        vault: vault_pda(&community),
        svm,
        admin,
        mint,
        fee_recipient,
        community,
    }
}

impl Env {
    fn balance(&self, key: &Pubkey) -> u64 {
        self.svm.get_balance(key).unwrap_or(0)
    }

    fn deposit(&mut self, lamports: u64) {
        let donor = funded(&mut self.svm, lamports + SOL);
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: system_program::ID,
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(donor.pubkey(), true),
                anchor_lang::solana_program::instruction::AccountMeta::new(self.vault, false),
            ],
            // System program Transfer: u32le variant 2, then u64le lamports.
            data: [2u32.to_le_bytes().as_slice(), &lamports.to_le_bytes()].concat(),
        };
        send(&mut self.svm, ix, &[&donor]).unwrap();
    }

    fn publish_ix(
        &self,
        admin: &Pubkey,
        fee_recipient: Pubkey,
        index: u64,
        root: [u8; 32],
        gross: u64,
        allocated: u64,
    ) -> Instruction {
        Instruction {
            program_id: hyphae::ID,
            accounts: hyphae::accounts::PublishEpoch {
                admin: *admin,
                community: self.community,
                vault: self.vault,
                fee_recipient,
                epoch: epoch_pda(&self.community, index),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: hyphae::instruction::PublishEpoch {
                index,
                root,
                audit_hash: [0xa0; 32],
                gross_lamports: gross,
                allocated_lamports: allocated,
            }
            .data(),
        }
    }

    fn publish(&mut self, index: u64, root: [u8; 32], gross: u64, allocated: u64) -> Outcome {
        let ix = self.publish_ix(
            &self.admin.pubkey(),
            self.fee_recipient,
            index,
            root,
            gross,
            allocated,
        );
        let admin = self.admin.insecure_clone();
        send(&mut self.svm, ix, &[&admin])
    }

    fn claim(
        &mut self,
        claimant: &Keypair,
        index: u64,
        leaf: &Leaf,
        amount: u64,
        proof: Vec<[u8; 32]>,
    ) -> Outcome {
        let epoch = epoch_pda(&self.community, index);
        let ix = Instruction {
            program_id: hyphae::ID,
            accounts: hyphae::accounts::Claim {
                claimant: claimant.pubkey(),
                community: self.community,
                vault: self.vault,
                epoch,
                receipt: receipt_pda(&epoch, &claimant.pubkey()),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: hyphae::instruction::Claim {
                score: leaf.score,
                amount,
                evidence_hash: leaf.evidence_hash,
                proof,
            }
            .data(),
        };
        send(&mut self.svm, ix, &[claimant])
    }

    fn community(&self) -> Community {
        let data = self.svm.get_account(&self.community).unwrap().data;
        Community::try_deserialize(&mut data.as_slice()).unwrap()
    }

    fn epoch(&self, index: u64) -> Epoch {
        let data = self
            .svm
            .get_account(&epoch_pda(&self.community, index))
            .unwrap()
            .data;
        Epoch::try_deserialize(&mut data.as_slice()).unwrap()
    }

    fn rent(&self, space: usize) -> u64 {
        self.svm.minimum_balance_for_rent_exemption(space)
    }
}

// A leaf of the shared vectors (the TS side built its tree and proofs), with its signing key.
struct Leaf {
    key: Keypair,
    score: u64,
    amount: u64,
    evidence_hash: [u8; 32],
    epoch_index: u64,
}

fn leaf(v: &Value, name: &str) -> Leaf {
    let l = v["merkle"]["leaves"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["name"] == name)
        .unwrap();
    let key = Keypair::new_from_array(hex32(&l["seed"]));
    assert_eq!(key.pubkey().to_bytes(), hex32(&l["wallet"]));
    Leaf {
        key,
        score: dec(&l["score"]),
        amount: dec(&l["amount"]),
        evidence_hash: hex32(&l["evidence_hash"]),
        epoch_index: dec(&l["epoch_index"]),
    }
}

// The payment proposal's worked example: 0.5 SOL gross, a 3% fee, and the three leaves a, b, c of
// the vectors' three-leaf tree (121,250,000 + 108,486,842 + 51,052,631 lamports).
const GROSS: u64 = 500_000_000;
const FEE: u64 = 15_000_000;
const ALLOCATED: u64 = 280_789_473;

fn published() -> (Env, Value) {
    let v = vectors();
    let mut env = setup();
    env.deposit(SOL);
    let root = hex32(&tree(&v, 3)["root"]);
    env.publish(2, root, GROSS, ALLOCATED).unwrap();
    (env, v)
}

fn claimant(env: &mut Env, leaf: &Leaf) -> Keypair {
    env.svm.airdrop(&leaf.key.pubkey(), SOL / 10).unwrap();
    leaf.key.insecure_clone()
}

#[test]
fn initialize_creates_the_community_and_its_program_owned_vault() {
    let env = setup();
    let c = env.community();
    assert_eq!(c.mint, env.mint);
    assert_eq!(c.admin, env.admin.pubkey());
    assert_eq!(c.fee_recipient, env.fee_recipient);
    assert_eq!(c.outstanding_lamports, 0);
    let vault = env.svm.get_account(&env.vault).unwrap();
    assert_eq!(vault.owner, hyphae::ID);
}

#[test]
fn initialize_refuses_an_account_that_is_not_a_token_mint() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        hyphae::ID,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/hyphae.so"),
    )
    .unwrap();
    let admin = funded(&mut svm, SOL);
    let wallet = funded(&mut svm, SOL).pubkey();
    // A token account is owned by the token program too, but it is not a mint.
    let token_account = owned_account(&mut svm, TOKEN_PROGRAM, vec![0; 165]);
    let token_2022_account = token_2022_with_extensions(&mut svm, 2);
    let uninitialized = owned_account(&mut svm, TOKEN_PROGRAM, vec![0; 82]);
    for not_a_mint in [wallet, token_account, token_2022_account, uninitialized] {
        let outcome = send(
            &mut svm,
            initialize_ix(&admin.pubkey(), &not_a_mint, Pubkey::new_unique()),
            &[&admin],
        );
        assert_eq!(custom(&outcome), Some(NOT_A_MINT));
    }
    // Token-2022 mints, with and without extensions, are mints.
    for mint in [
        mint_account(&mut svm, TOKEN_2022_PROGRAM),
        token_2022_with_extensions(&mut svm, 1),
    ] {
        send(
            &mut svm,
            initialize_ix(&admin.pubkey(), &mint, Pubkey::new_unique()),
            &[&admin],
        )
        .unwrap();
    }
}

#[test]
fn initialize_refuses_a_fee_recipient_that_cannot_receive_fees() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        hyphae::ID,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/hyphae.so"),
    )
    .unwrap();
    let admin = funded(&mut svm, SOL);
    let mint = mint_account(&mut svm, TOKEN_PROGRAM);
    let outcome = send(
        &mut svm,
        initialize_ix(&admin.pubkey(), &mint, Pubkey::default()),
        &[&admin],
    );
    assert_eq!(custom(&outcome), Some(INVALID_FEE_RECIPIENT));
    // The vault itself cannot be the fee recipient: a fee paid to it would never leave.
    let vault = vault_pda(&community_pda(&mint, &admin.pubkey()));
    let outcome = send(
        &mut svm,
        initialize_ix(&admin.pubkey(), &mint, vault),
        &[&admin],
    );
    assert_eq!(custom(&outcome), Some(INVALID_FEE_RECIPIENT));
}

#[test]
fn publish_moves_the_floored_3_percent_fee_to_the_recorded_recipient() {
    let v = vectors();
    let mut env = setup();
    env.deposit(SOL);
    let vault_before = env.balance(&env.vault);
    let fee_before = env.balance(&env.fee_recipient);
    let root = hex32(&tree(&v, 3)["root"]);

    env.publish(2, root, GROSS, ALLOCATED).unwrap();

    assert_eq!(env.balance(&env.fee_recipient) - fee_before, FEE);
    assert_eq!(vault_before - env.balance(&env.vault), FEE);
    let e = env.epoch(2);
    assert_eq!(e.community, env.community);
    assert_eq!(e.index, 2);
    assert_eq!(e.root, root);
    assert_eq!(e.audit_hash, [0xa0; 32]);
    assert_eq!(e.gross_lamports, GROSS);
    assert_eq!(e.fee_lamports, FEE);
    assert_eq!(e.allocated_lamports, ALLOCATED);
    assert_eq!(e.claimed_lamports, 0);
    assert_eq!(env.community().outstanding_lamports, ALLOCATED);
}

#[test]
fn publish_floors_the_fee() {
    let mut env = setup();
    env.deposit(SOL);
    let fee_before = env.balance(&env.fee_recipient);
    // 333 × 300 / 10,000 = 9.99, floored to 9 in the contributors' favour (P7).
    env.publish(2, [7; 32], 333, 324).unwrap();
    assert_eq!(env.balance(&env.fee_recipient) - fee_before, 9);
    assert_eq!(env.epoch(2).fee_lamports, 9);
}

#[test]
fn publish_refuses_anyone_but_the_admin() {
    let mut env = setup();
    env.deposit(SOL);
    let stranger = funded(&mut env.svm, SOL);
    let ix = env.publish_ix(&stranger.pubkey(), env.fee_recipient, 2, [7; 32], GROSS, 1);
    let outcome = send(&mut env.svm, ix, &[&stranger]);
    assert_eq!(custom(&outcome), Some(CONSTRAINT_SEEDS));
    assert!(env.svm.get_account(&epoch_pda(&env.community, 2)).is_none());
}

#[test]
fn publish_refuses_another_fee_recipient() {
    let mut env = setup();
    env.deposit(SOL);
    let other = funded(&mut env.svm, SOL).pubkey();
    let ix = env.publish_ix(&env.admin.pubkey(), other, 2, [7; 32], GROSS, 1);
    let admin = env.admin.insecure_clone();
    let outcome = send(&mut env.svm, ix, &[&admin]);
    assert_eq!(custom(&outcome), Some(CONSTRAINT_ADDRESS));
}

#[test]
fn publish_refuses_a_pot_above_the_unassigned_balance() {
    let mut env = setup();
    env.deposit(SOL);
    // The vault holds 1 SOL plus its own rent; the rent is never part of a pot.
    let available = env.balance(&env.vault) - env.rent(8 + 1);
    assert_eq!(available, SOL);
    assert_eq!(
        custom(&env.publish(2, [7; 32], available + 1, 1)),
        Some(INSUFFICIENT_VAULT_BALANCE)
    );
    env.publish(2, [7; 32], available, 1).unwrap();
}

#[test]
fn allocated_lamports_stay_reserved_for_later_epochs() {
    let mut env = setup();
    env.deposit(SOL);
    env.publish(2, [7; 32], GROSS, ALLOCATED).unwrap();
    // Unassigned now: 1 SOL − fee − allocated. Cap remainder and dust stayed unassigned.
    let unassigned = SOL - FEE - ALLOCATED;
    assert_eq!(
        custom(&env.publish(3, [8; 32], unassigned + 1, 1)),
        Some(INSUFFICIENT_VAULT_BALANCE)
    );
    env.publish(3, [8; 32], unassigned, 1).unwrap();
}

#[test]
fn publish_refuses_an_allocation_of_zero_or_above_the_net_pot() {
    let mut env = setup();
    env.deposit(SOL);
    assert_eq!(
        custom(&env.publish(2, [7; 32], GROSS, 0)),
        Some(INVALID_ALLOCATION)
    );
    assert_eq!(
        custom(&env.publish(2, [7; 32], GROSS, GROSS - FEE + 1)),
        Some(INVALID_ALLOCATION)
    );
    assert_eq!(custom(&env.publish(2, [7; 32], 0, 0)), Some(ZERO_GROSS));
    assert_eq!(custom(&env.publish(2, [0; 32], GROSS, 1)), Some(EMPTY_ROOT));
    env.publish(2, [7; 32], GROSS, GROSS - FEE).unwrap();
}

#[test]
fn an_epoch_is_published_once() {
    let mut env = setup();
    env.deposit(SOL);
    env.publish(2, [7; 32], GROSS / 4, 1).unwrap();
    assert!(env.publish(2, [9; 32], GROSS / 4, 1).is_err());
    assert_eq!(env.epoch(2).root, [7; 32]);
}

#[test]
fn a_claim_pays_the_leaf_amount_once_and_writes_a_receipt() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    let wallet_before = env.balance(&key.pubkey());
    let vault_before = env.balance(&env.vault);

    env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"))
        .unwrap();

    assert_eq!(vault_before - env.balance(&env.vault), a.amount);
    let receipt_rent = env.rent(8 + 32 + 32 + 8 + 8 + 32 + 8 + 1);
    assert_eq!(
        env.balance(&key.pubkey()),
        wallet_before + a.amount - receipt_rent - TX_FEE
    );
    let epoch = epoch_pda(&env.community, 2);
    let data = env
        .svm
        .get_account(&receipt_pda(&epoch, &key.pubkey()))
        .unwrap()
        .data;
    let r = ClaimReceipt::try_deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(r.epoch, epoch);
    assert_eq!(r.wallet, key.pubkey());
    assert_eq!(r.amount, a.amount);
    assert_eq!(r.score, a.score);
    assert_eq!(r.evidence_hash, a.evidence_hash);
    assert_eq!(env.epoch(2).claimed_lamports, a.amount);
    assert_eq!(env.community().outstanding_lamports, ALLOCATED - a.amount);
}

#[test]
fn a_second_claim_of_the_same_leaf_fails() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"))
        .unwrap();
    let vault_after_first = env.balance(&env.vault);

    let second = env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"));

    assert!(second.is_err());
    assert_eq!(env.balance(&env.vault), vault_after_first);
    assert_eq!(env.epoch(2).claimed_lamports, a.amount);
}

#[test]
fn every_leaf_of_the_tree_can_claim_and_the_epoch_ends_fully_claimed() {
    let (mut env, v) = published();
    for name in ["a", "b", "c"] {
        let l = leaf(&v, name);
        let key = claimant(&mut env, &l);
        env.claim(&key, 2, &l, l.amount, proof(tree(&v, 3), name))
            .unwrap();
    }
    assert_eq!(env.epoch(2).claimed_lamports, ALLOCATED);
    assert_eq!(env.community().outstanding_lamports, 0);
}

#[test]
fn a_wrong_proof_fails() {
    let (mut env, v) = published();
    let b = leaf(&v, "b");
    let key = claimant(&mut env, &b);
    let tampered: Vec<[u8; 32]> = v["merkle"]["tampered"][0]["proof"]
        .as_array()
        .unwrap()
        .iter()
        .map(hex32)
        .collect();
    assert_eq!(
        custom(&env.claim(&key, 2, &b, b.amount, tampered)),
        Some(INVALID_PROOF)
    );
    // Another leaf's proof fails too.
    assert_eq!(
        custom(&env.claim(&key, 2, &b, b.amount, proof(tree(&v, 3), "a"))),
        Some(INVALID_PROOF)
    );
}

#[test]
fn a_claim_for_more_than_the_leaf_fails() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    assert_eq!(
        custom(&env.claim(&key, 2, &a, a.amount + 1, proof(tree(&v, 3), "a"))),
        Some(INVALID_PROOF)
    );
}

#[test]
fn someone_else_cannot_claim_a_leaf() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let thief = funded(&mut env.svm, SOL);
    assert_eq!(
        custom(&env.claim(&thief, 2, &a, a.amount, proof(tree(&v, 3), "a"))),
        Some(INVALID_PROOF)
    );
}

#[test]
fn a_leaf_only_claims_against_its_own_epoch() {
    let (mut env, v) = published();
    // The same root published again as epoch 3: the leaf commits to epoch index 2.
    env.publish(3, hex32(&tree(&v, 3)["root"]), GROSS / 2, 1)
        .unwrap();
    let a = leaf(&v, "a");
    assert_eq!(a.epoch_index, 2);
    let key = claimant(&mut env, &a);
    assert_eq!(
        custom(&env.claim(&key, 3, &a, a.amount, proof(tree(&v, 3), "a"))),
        Some(INVALID_PROOF)
    );
    env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"))
        .unwrap();
}

#[test]
fn a_claim_before_publish_fails() {
    let v = vectors();
    let mut env = setup();
    env.deposit(SOL);
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    assert_eq!(
        custom(&env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"))),
        Some(ACCOUNT_NOT_INITIALIZED)
    );
}

#[test]
fn claims_never_exceed_the_epoch_allocation() {
    let v = vectors();
    let mut env = setup();
    env.deposit(SOL);
    let a = leaf(&v, "a");
    let b = leaf(&v, "b");
    // A root whose leaves add up to more than the published allocation.
    env.publish(2, hex32(&tree(&v, 3)["root"]), GROSS, a.amount)
        .unwrap();
    let key_a = claimant(&mut env, &a);
    env.claim(&key_a, 2, &a, a.amount, proof(tree(&v, 3), "a"))
        .unwrap();
    let key_b = claimant(&mut env, &b);
    assert_eq!(
        custom(&env.claim(&key_b, 2, &b, b.amount, proof(tree(&v, 3), "b"))),
        Some(EPOCH_OVERCLAIMED)
    );
}

#[test]
fn a_zero_claim_fails() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    assert_eq!(
        custom(&env.claim(&key, 2, &a, 0, proof(tree(&v, 3), "a"))),
        Some(ZERO_AMOUNT)
    );
}

// Anyone can send lamports to a future PDA, which leaves an empty System-owned account there.
// Anchor's `init` still creates each account, so prefunding cannot block a community, an epoch
// or a claim.
#[test]
fn initialize_creates_a_community_and_vault_someone_prefunded() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        hyphae::ID,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/hyphae.so"),
    )
    .unwrap();
    let admin = funded(&mut svm, 10 * SOL);
    let mint = mint_account(&mut svm, TOKEN_PROGRAM);
    let community = community_pda(&mint, &admin.pubkey());
    svm.airdrop(&community, 1_000_000).unwrap();
    svm.airdrop(&vault_pda(&community), 1_000_000).unwrap();
    let fee_recipient = funded(&mut svm, SOL).pubkey();

    send(
        &mut svm,
        initialize_ix(&admin.pubkey(), &mint, fee_recipient),
        &[&admin],
    )
    .unwrap();

    assert_eq!(svm.get_account(&community).unwrap().owner, hyphae::ID);
    assert_eq!(
        svm.get_account(&vault_pda(&community)).unwrap().owner,
        hyphae::ID
    );
}

#[test]
fn publish_creates_an_epoch_someone_prefunded() {
    let v = vectors();
    let mut env = setup();
    env.deposit(SOL);
    let epoch = epoch_pda(&env.community, 2);
    env.svm.airdrop(&epoch, 1_000_000).unwrap();
    let root = hex32(&tree(&v, 3)["root"]);

    env.publish(2, root, GROSS, ALLOCATED).unwrap();

    assert_eq!(env.svm.get_account(&epoch).unwrap().owner, hyphae::ID);
    assert_eq!(env.epoch(2).root, root);
    assert_eq!(env.community().outstanding_lamports, ALLOCATED);
}

#[test]
fn a_claim_creates_a_receipt_someone_prefunded() {
    let (mut env, v) = published();
    let a = leaf(&v, "a");
    let key = claimant(&mut env, &a);
    let receipt = receipt_pda(&epoch_pda(&env.community, 2), &key.pubkey());
    env.svm.airdrop(&receipt, 1_000_000).unwrap();
    let vault_before = env.balance(&env.vault);

    env.claim(&key, 2, &a, a.amount, proof(tree(&v, 3), "a"))
        .unwrap();

    assert_eq!(env.svm.get_account(&receipt).unwrap().owner, hyphae::ID);
    assert_eq!(vault_before - env.balance(&env.vault), a.amount);
    assert_eq!(env.epoch(2).claimed_lamports, a.amount);
}
