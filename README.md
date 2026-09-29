# Hyphae program

Hyphae is a proof-of-contribution layer for token communities. Members do real work for a community, an AI scores it against public guidelines and shows its reasoning, and each epoch's payouts are committed to Solana as a merkle root, next to the hash of the epoch's full audit record. Each community has a SOL vault; a contributor claims their share from it with a merkle proof, signed by their own wallet, once.

This repository holds the part you have to trust with money: **the Solana program**, and **the rubrics** every contribution is graded against. Both are public so anyone can rebuild the program and check it against the chain, and read the rules before doing the work. The rest of Hyphae (the Telegram bot, scoring, the read API and the site) is developed privately.

- Site: **[hyphae-delta.vercel.app](https://hyphae-delta.vercel.app)**
- Built solo for [Colosseum's Crypto World's Fair](https://colosseum.com/worldsfair) (2026-09-14 → 2026-10-12).

## Status

| Part | State |
|---|---|
| Program (`programs/hyphae`): per-community vaults, epoch roots, one-time claims | Deployed on **devnet** only, at `EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E`. Not on mainnet. |
| Rubrics (`rubrics/`) | MYCEL 1.2.0 scores live; 1.3.0 is a candidate, not applied. |

## Verify the build

The program builds reproducibly in Anchor's pinned Docker image, `solanafoundation/anchor:v1.0.1` (digest `sha256:afa0c004f1ec96b5f420990dda46140298de00c528d177b62d6841242d2db0be`). Linux or WSL, with Docker, Anchor 1.0.1 and `solana-verify`:

```sh
git clone https://github.com/FCisco95/hyphae-program && cd hyphae-program/programs/hyphae
anchor build --verifiable --ignore-keys
sha256sum ../../target/verifiable/hyphae.so
solana-verify get-executable-hash ../../target/verifiable/hyphae.so
solana-verify get-program-hash -u devnet EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E
```

| What | Hash |
|---|---|
| `hyphae.so` sha256 (229,432 bytes) | `cb4ffdd8074442310f7953b5233a4c2df4eaf8c3f7627cdf259efb84ebf98d79` |
| `solana-verify` executable hash, and the devnet program's on-chain hash | `7e902d1b5f8d8c49dfd199ec2e7bf44139b56524d98408f1556e14f4e9ab43ac` |

`--ignore-keys`: a fresh clone has no program keypair, and `declare_id!` already pins the address. `solana-verify` hashes the program without its trailing zero padding, so its hash is the one to compare with the chain.

The program's tests run on LiteSVM against the built program:

```sh
anchor build && cargo test -p hyphae --tests
```

They read the shared commitment vectors in `packages/core/src/test-vectors/h-contract-v1.json`, which the private TypeScript implementation also checks itself against.

## A full run on devnet

Hyphae's admin key on a Ledger signs the community, the deposit and the publication; a member claims, then is refused.

| Step | Transaction |
|---|---|
| A community and its vault are created, signed by Hyphae's admin key on a Ledger. | [`2xkfBYCq…vsENo`](https://explorer.solana.com/tx/2xkfBYCqiJhQupUL6gB7P9m6DkpgomVkysYMw5bRVveAZvBDmSwzZ7C3kfSVDpaY5PFoN7mcwXbQ8KPHhStvsENo?cluster=devnet) |
| The vault is funded with 0.05 SOL. | [`2Af95ffY…aaoof`](https://explorer.solana.com/tx/2Af95ffYBU6fE11G7hJryYeASkD85HsZ1ikfbbHu1srBp72hWHSzaJRAiqZuRVj65EFxoYJQgRu6Z3dKb8agaoof?cluster=devnet) |
| The epoch is published as one merkle root: 0.030460365 SOL allocated to 3 members, and the 3% fee. | [`3oJ4T6Ne…gy6DD`](https://explorer.solana.com/tx/3oJ4T6NeYonVRgi1JBfofsRHEkd5RfuTKaEhUc7AHqFL8Pio2r2s6o4n7YtLm7zMPpLCw38mDa1efEzBBpAgy6DD?cluster=devnet) |
| A member claims 0.012125 SOL with a merkle proof. | [`5ccGT1yL…a3SmK`](https://explorer.solana.com/tx/5ccGT1yLxySoRKjUZftCooXmbxk35WrJ71XFVuH8rfXxaKNiufPkzgLRdAbywRoLjoTL4DvFv3daYTgGyp8a3SmK?cluster=devnet) |
| The same claim, sent again, is refused on-chain. | [`5ob9A3Sg…2EknP`](https://explorer.solana.com/tx/5ob9A3Sg7DYoxCAMLDyX5EpuT2jSF6QBsruPRLf4tdTdTCSkfZCdXSkMuWLuqkFEPQC3eQJVGgzosxdLQbA2EknP?cluster=devnet) |

## Read API

Public, read-only, unauthenticated JSON at `https://hyphae-api.fly.dev/v1`. Production serves `/v1/communities/{mint}` and `/v1/communities/{mint}/epochs/{index}` today; the wallet-claims route below, the reference at `/docs` and the OpenAPI 3.1 document at `/v1/openapi.json` are built and tested, and not deployed yet.

- A section the API cannot confirm is `{ "status": "unavailable", "reason": … }`, never a zero.
- Settlement and payments are read against Solana. A transaction is shown only when the chain proves it created the account it names.
- Every response carries `RateLimit-*` headers. Past 300 requests a minute from one address, the API answers `429` with `Retry-After`.

## Check a proof yourself

A wallet's leaves in every community, newest first, each with its proof and its payment status (`paid` with the claim transaction, `claimable`, or `unavailable` with a reason):

```js
const API = process.env.HYPHAE_API ?? "https://hyphae-api.fly.dev/v1";
const wallet = process.argv[2];
const { claims } = await (await fetch(`${API}/wallets/${wallet}/claims?limit=100`)).json();
```

Rebuild the 89-byte leaf, then hash up the sorted pairs to the root. It is the same computation the program runs before it pays (`programs/hyphae/src/merkle.rs`).

```js
import { createHash } from "node:crypto";
import { getAddressEncoder } from "@solana/kit";

const sha = (...parts) => createHash("sha256").update(Buffer.concat(parts)).digest();
const u64 = (v) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(v)); return b; };
for (const c of claims) {
  let node = sha(Buffer.of(0), Buffer.from(getAddressEncoder().encode(wallet)), u64(c.epoch.index),
    u64(c.score), u64(c.amount_lamports), Buffer.from(c.evidence_hash, "hex"));
  for (const p of c.proof.map((h) => Buffer.from(h, "hex")))
    node = sha(Buffer.of(1), ...(Buffer.compare(node, p) <= 0 ? [node, p] : [p, node]));
  console.log(c.community.mint, c.epoch.index, c.amount_lamports, c.payment.status, node.toString("hex") === c.root);
}
```

`root` is the root of the epoch account at `epoch_address`. Read that account to check it without trusting the API.

## Funding a community's vault

A community is the pair (token mint, the admin key that publishes its epochs). Its vault is a program-derived address, so an integrator derives it without calling Hyphae, and funds it with an ordinary SOL transfer:

```js
import { address, getAddressEncoder, getProgramDerivedAddress } from "@solana/kit";

const HYPHAE = address("EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E");
const enc = getAddressEncoder();
const [community] = await getProgramDerivedAddress({ programAddress: HYPHAE, seeds: ["community", enc.encode(mint), enc.encode(admin)] });
const [vault] = await getProgramDerivedAddress({ programAddress: HYPHAE, seeds: ["vault", enc.encode(community)] });
```

- SOL leaves the vault only through the program: each published epoch sends the 3% Hyphae fee to the recipient fixed when the community was created, and each claim pays one leaf of a published root, once.
- An epoch can only allocate SOL that no earlier epoch has allocated and nobody has claimed yet.
- The program is on devnet only. Check its address on the network you use before sending anything.

## Custody during the pilot

**Pilot policy.** Hyphae's publisher key sets each epoch's payout list, so you trust it with that epoch's pot. We keep that key on a hardware wallet and fund one epoch at a time, just before it pays. The program has no withdraw instruction: SOL leaves the vault only through member claims and the 3% fee. The program can still be upgraded. The upgrade key is held the same way, and any upgrade is announced here before it is used.

## License

[Business Source License 1.1](LICENSE). You may read, build, audit and verify this code, and use it to interact with the Hyphae programs and services the Licensor runs. Running it, or a derivative, as your own program or service needs a commercial license until the Change Date, 2028-10-12, when it becomes available under the GPL v2.0 or later.
