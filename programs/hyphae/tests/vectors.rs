mod common;

use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountSerialize, Discriminator, InstructionData, ToAccountMetas};
use common::{dec, hex, hex32, proof, tree, vectors};
use hyphae::merkle::{encode_leaf, hash_pair, leaf_hash, verify_proof};
use hyphae::state::{ClaimReceipt, Community, Epoch, Vault};
use serde_json::Value;

fn leaf_of(l: &Value) -> (Pubkey, u64, u64, u64, [u8; 32]) {
    (
        Pubkey::new_from_array(hex32(&l["wallet"])),
        dec(&l["epoch_index"]),
        dec(&l["score"]),
        dec(&l["amount"]),
        hex32(&l["evidence_hash"]),
    )
}

#[test]
fn leaf_bytes_and_hashes_match_typescript() {
    let v = vectors();
    for l in v["merkle"]["leaves"].as_array().unwrap() {
        let (wallet, index, score, amount, evidence) = leaf_of(l);
        assert_eq!(
            encode_leaf(&wallet, index, score, amount, &evidence).to_vec(),
            hex(l["encoded"].as_str().unwrap()),
            "leaf {}",
            l["name"]
        );
        assert_eq!(
            leaf_hash(&wallet, index, score, amount, &evidence),
            hex32(&l["hash"]),
            "leaf {}",
            l["name"]
        );
    }
}

#[test]
fn roots_and_proofs_match_typescript() {
    let v = vectors();
    let leaves = v["merkle"]["leaves"].as_array().unwrap();
    let hash_of = |name: &str| hex32(&leaves.iter().find(|l| l["name"] == name).unwrap()["hash"]);
    for t in v["merkle"]["trees"].as_array().unwrap() {
        let root = hex32(&t["root"]);
        let names: Vec<&str> = t["leaves"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        for name in &names {
            assert!(
                verify_proof(&root, hash_of(name), &proof(t, name)),
                "{name} in the tree of {}",
                names.len()
            );
        }
        if names.len() == 2 {
            assert_eq!(hash_pair(&hash_of(names[0]), &hash_of(names[1])), root);
            assert_eq!(hash_pair(&hash_of(names[1]), &hash_of(names[0])), root);
        }
    }
}

#[test]
fn a_tampered_proof_fails() {
    let v = vectors();
    let leaves = v["merkle"]["leaves"].as_array().unwrap();
    for t in v["merkle"]["tampered"].as_array().unwrap() {
        let root = hex32(&tree(&v, t["tree"].as_u64().unwrap() as usize)["root"]);
        let leaf = leaves.iter().find(|l| l["name"] == t["leaf"]).unwrap();
        let proof: Vec<[u8; 32]> = t["proof"].as_array().unwrap().iter().map(hex32).collect();
        assert_eq!(
            verify_proof(&root, hex32(&leaf["hash"]), &proof),
            t["valid"].as_bool().unwrap()
        );
    }
}

#[test]
fn instruction_bytes_match_typescript() {
    let v = vectors();
    let ix = &v["program"]["instructions"];
    let disc = |name: &str| hex(ix[name]["discriminator"].as_str().unwrap());
    let data = |name: &str| hex(ix[name]["data"].as_str().unwrap());

    let a = &ix["initialize_community"]["args"];
    assert_eq!(
        hyphae::instruction::InitializeCommunity::DISCRIMINATOR,
        disc("initialize_community")
    );
    assert_eq!(
        hyphae::instruction::InitializeCommunity {
            fee_recipient: Pubkey::new_from_array(hex32(&a["fee_recipient"])),
        }
        .data(),
        data("initialize_community")
    );

    let a = &ix["publish_epoch"]["args"];
    assert_eq!(
        hyphae::instruction::PublishEpoch::DISCRIMINATOR,
        disc("publish_epoch")
    );
    assert_eq!(
        hyphae::instruction::PublishEpoch {
            index: dec(&a["index"]),
            root: hex32(&a["root"]),
            audit_hash: hex32(&a["audit_hash"]),
            gross_lamports: dec(&a["gross_lamports"]),
            allocated_lamports: dec(&a["allocated_lamports"]),
        }
        .data(),
        data("publish_epoch")
    );

    let a = &ix["claim"]["args"];
    assert_eq!(hyphae::instruction::Claim::DISCRIMINATOR, disc("claim"));
    assert_eq!(
        hyphae::instruction::Claim {
            score: dec(&a["score"]),
            amount: dec(&a["amount"]),
            evidence_hash: hex32(&a["evidence_hash"]),
            proof: a["proof"].as_array().unwrap().iter().map(hex32).collect(),
        }
        .data(),
        data("claim")
    );
}

#[test]
fn account_order_and_roles_match_typescript() {
    let v = vectors();
    let ix = &v["program"]["instructions"];
    let expected = |name: &str| -> Vec<(bool, bool)> {
        ix[name]["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                (
                    a["writable"].as_bool().unwrap(),
                    a["signer"].as_bool().unwrap(),
                )
            })
            .collect()
    };
    let roles =
        |metas: Vec<anchor_lang::solana_program::instruction::AccountMeta>| -> Vec<(bool, bool)> {
            metas.iter().map(|m| (m.is_writable, m.is_signer)).collect()
        };
    let k = || Pubkey::new_unique();

    assert_eq!(
        roles(
            hyphae::accounts::InitializeCommunity {
                admin: k(),
                mint: k(),
                community: k(),
                vault: k(),
                system_program: k(),
            }
            .to_account_metas(None)
        ),
        expected("initialize_community")
    );
    assert_eq!(
        roles(
            hyphae::accounts::PublishEpoch {
                admin: k(),
                community: k(),
                vault: k(),
                fee_recipient: k(),
                epoch: k(),
                system_program: k(),
            }
            .to_account_metas(None)
        ),
        expected("publish_epoch")
    );
    assert_eq!(
        roles(
            hyphae::accounts::Claim {
                claimant: k(),
                community: k(),
                vault: k(),
                epoch: k(),
                receipt: k(),
                system_program: k(),
            }
            .to_account_metas(None)
        ),
        expected("claim")
    );
}

fn serialized<T: AccountSerialize>(account: &T) -> Vec<u8> {
    let mut out = Vec::new();
    account.try_serialize(&mut out).unwrap();
    out
}

#[test]
fn account_bytes_match_typescript() {
    let v = vectors();
    let acc = &v["program"]["accounts"];
    let pk = |x: &Value| Pubkey::new_from_array(hex32(x));
    let byte = |x: &Value| x.as_u64().unwrap() as u8;
    let disc = |name: &str| hex(acc[name]["discriminator"].as_str().unwrap());
    let data = |name: &str| hex(acc[name]["data"].as_str().unwrap());

    let c = &acc["community"]["fields"];
    assert_eq!(Community::DISCRIMINATOR, disc("community"));
    assert_eq!(
        serialized(&Community {
            mint: pk(&c["mint"]),
            admin: pk(&c["admin"]),
            fee_recipient: pk(&c["fee_recipient"]),
            outstanding_lamports: dec(&c["outstanding_lamports"]),
            bump: byte(&c["bump"]),
            vault_bump: byte(&c["vault_bump"]),
        }),
        data("community")
    );

    assert_eq!(Vault::DISCRIMINATOR, disc("vault"));
    assert_eq!(
        serialized(&Vault {
            bump: byte(&acc["vault"]["fields"]["bump"]),
        }),
        data("vault")
    );

    let e = &acc["epoch"]["fields"];
    assert_eq!(Epoch::DISCRIMINATOR, disc("epoch"));
    assert_eq!(
        serialized(&Epoch {
            community: pk(&e["community"]),
            index: dec(&e["index"]),
            root: hex32(&e["root"]),
            audit_hash: hex32(&e["audit_hash"]),
            gross_lamports: dec(&e["gross_lamports"]),
            fee_lamports: dec(&e["fee_lamports"]),
            allocated_lamports: dec(&e["allocated_lamports"]),
            claimed_lamports: dec(&e["claimed_lamports"]),
            published_at: e["published_at"].as_str().unwrap().parse().unwrap(),
            bump: byte(&e["bump"]),
        }),
        data("epoch")
    );

    let r = &acc["claim_receipt"]["fields"];
    assert_eq!(ClaimReceipt::DISCRIMINATOR, disc("claim_receipt"));
    assert_eq!(
        serialized(&ClaimReceipt {
            epoch: pk(&r["epoch"]),
            wallet: pk(&r["wallet"]),
            score: dec(&r["score"]),
            amount: dec(&r["amount"]),
            evidence_hash: hex32(&r["evidence_hash"]),
            claimed_at: r["claimed_at"].as_str().unwrap().parse().unwrap(),
            bump: byte(&r["bump"]),
        }),
        data("claim_receipt")
    );
}
