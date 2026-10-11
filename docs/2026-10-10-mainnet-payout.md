# October 10 mainnet payout receipts

Epoch 2 was published on 2026-10-10 and all four payees claimed that day. Three payees are non-founder community members; the fourth is the founder. The operator attested their X accounts. The program proves wallet signatures, not X account ownership.

## Accounts and allocation

- Program: [EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E](https://solscan.io/account/EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E); mainnet upgrade authority: Ledger `2kz1Zq8UDm9Hq6XwPW6cViQZe7aySEBGk1gLWN8gofjR`.
- MYCEL community: [HRkBN4sX7NyPEfa4SfRoTsP1dynmPDLMYbY7qLa4XbRX](https://solscan.io/account/HRkBN4sX7NyPEfa4SfRoTsP1dynmPDLMYbY7qLa4XbRX); vault: [AC3zkGQ9abJs6sssaY5nDX8Qjv2UM19r4JYLgcHoG86K](https://solscan.io/account/AC3zkGQ9abJs6sssaY5nDX8Qjv2UM19r4JYLgcHoG86K).
- Epoch 2: [J7ipBhK2eJu8QFGtXTsYWNDhPYzcerwX22UkkJTaCPXG](https://solscan.io/account/J7ipBhK2eJu8QFGtXTsYWNDhPYzcerwX22UkkJTaCPXG); [live settlement](https://hyphae-delta.vercel.app/c/HudkzEWpcUnTYFZMMcbNdwk1S5Am26J2SyEh4NfFworg/e/2).
- Gross: 1,500,000,000 lamports; Treasury fee: 45,000,000; allocated and claimed: **1,238,881,509**; unclaimed: **0**. Four of four claims are paid. Cap remainder: 216,118,490; dust: 1. These are allocation amounts, not the vault's rent-inclusive balance.
- [Public epoch-2 API](https://hyphae-api.fly.dev/v1/communities/HudkzEWpcUnTYFZMMcbNdwk1S5Am26J2SyEh4NfFworg/epochs/2): root `f98ff9300dc769691e797ec3c5a9f53d9c2082d2cb724d87b6077c49f5f7d1d3`; audit hash `c340a280f881781b18a6642b874708154df703816d94e37e13cc18a8633e0a90`.

## Finalized transactions

| Step | Receipt | Result |
|---|---|---|
| C16 | [aensGhPg…aQJSghNX](https://explorer.solana.com/tx/aensGhPgR8XBz5unAugUd9bJai3WbyUoJFwCLKmGn5kqUjsTNYNSQXR9oraAx9FitJac74tK9fYyCTBaQJSghNX) | Community and vault created with the Ledger admin and fixed Treasury fee recipient. |
| C18 | [5spBxwDa…i1c7rjtj](https://explorer.solana.com/tx/5spBxwDa9jbMbvZup3bR1aU5t2HVoYb24U2JLvX4dx8U6GJehDr1i1pgepxqTtodJf2ykRTP95dHGMBqi1c7rjtj) | 1.5 SOL transferred from the founder's funding wallet to the vault. |
| C20 | [4qhKtnsV…Xop6EnLm](https://explorer.solana.com/tx/4qhKtnsVD78xpvRPdi7TQ1fcF86FjiueCFZNq6a11yfSQ7Q4UeXboGaRtzTDtyMnFbDtVywHGzgytrXqXop6EnLm) | Epoch 2 published; 1,238,881,509 lamports allocated to four wallets; 0.045 SOL Treasury fee. |
| C21, founder | [5iBbZVJE…vD6sZn55](https://explorer.solana.com/tx/5iBbZVJEnjr9tvpHZ2dFYgsKAxy5tSvqQ3bUCaXY4fDGgH9PfomphF8SuTFrFqvpbFdLLE89AHZSSQRUvD6sZn55) | 363750000 lamports to `MAoRn1JFD66gcXHiCqcAgppXBCR4TP8pKT4XV7HVhAB`; [decoded claim receipt](https://solscan.io/account/C7nEMEavZm9BE8i2jFVK3Lq1sWA1TKTqtHws9iUxMDP9). |
| Member claim 1 | [2gbAmxM6…Wh4ek1tK](https://explorer.solana.com/tx/2gbAmxM6Q4LPC8R68wRbRyGKn7nD97vRZJ4FVD8eZucJab82LtDGFB1oK3d2TnzkkXM2m45aG6EpQc8QWh4ek1tK) | 223351074 lamports to `GsirkxvyAaNsvDCXFUJBh7vf3cMpv5X1wmBTwvHeXgt3`; [decoded claim receipt](https://solscan.io/account/Hti2rp3dNmzkdQjWoHxcYSa58tJc2b6D5my2hqmVrioa). |
| Member claim 2 | [59kNeiZ5…nGtAZvSy](https://explorer.solana.com/tx/59kNeiZ53TqEk9RU5Ujr5Wb6SfkjaRwtXBdBeovR9ZDPQC6H3qcRCd6YyspLDPwXGMmCmLxeZM99bcBsnGtAZvSy) | 363750000 lamports to `2BsmXzsH9fjDVTmkR2hGXfRPFcfv7EHvvF6JdjotneB8`; [decoded claim receipt](https://solscan.io/account/7NGcK4KiMG12Jfre5xBQq1BEHPtD34mMEGrDzTx8NYt). |
| Member claim 3 | [63tutuXC…vWRLCHzq](https://explorer.solana.com/tx/63tutuXCGdRvHrMZBaF973jDgQoweV5RFRCkmmP9xRG51xga9Xe7dDuAY78YbZA9EZR8WiRc92X88YoDvWRLCHzq) | 288030435 lamports to `4a5G78Rns6g7EUEmRw15Rk6NwNsUeZnQjPDgHbhdigvN`; [decoded claim receipt](https://solscan.io/account/3JfF36qcrxFpm2LgV6dmha2sSvLz7jmiWd82VwnGvGsS). |
| IDL initialization | [4Dp1vDjs…ZqSUEm7f](https://explorer.solana.com/tx/4Dp1vDjss1Rqcecaoy7UktDcWJe3nm3FC5ba5GkdWYnpq9RdXGYDCU7ZkNYtAchvsdtDTqpN9EERJpVvZqSUEm7f) | Canonical Program Metadata initialized. |
| IDL publication | [2sS66trK…aBj9ArgZ](https://explorer.solana.com/tx/2sS66trKN2myWW4cKSX2Yie3K2G9dDhqgE84aWvr9aZrm5R3hDpCmewFxN8GXzAj8kxYtYrNE8hw2F9haBj9ArgZ) | Canonical IDL at [zxj1hrV2pYRuHC65TvUVsRssMUvzu8bZ7YKWkPTrc9d](https://solscan.io/account/zxj1hrV2pYRuHC65TvUVsRssMUvzu8bZ7YKWkPTrc9d); Solscan decodes claim receipts and the epoch account. |

C22/P14 was the API and page readback after C21, not a separate transaction. The final October 10 read found 4/4 paid. A fresh public API read at 2026-10-11T09:37:31.133000Z returned the same totals. All nine transactions above were rechecked as finalized with no error on October 11.

## Program hash re-read, October 11

WSL `solana-verify` read the deployed mainnet executable through a public mainnet RPC:

```sh
solana-verify get-program-hash -u https://solana-rpc.publicnode.com EAz8WkyUbGqr3ewSLpk94GWEoiWsvMENE5zV7Tvh4d6E
# 7e902d1b5f8d8c49dfd199ec2e7bf44139b56524d98408f1556e14f4e9ab43ac
```

This matches the October 2 mainnet deployment and recorded reproducible build. The default mainnet-beta RPC failed from WSL; this alternate mainnet endpoint succeeded. This is a fresh chain hash read, not a fresh rebuild or an explorer verification registration.
