# TipidCircle

> Paluwagan on Stellar — the Filipino rotating savings group, now trustless.

---

## Problem

A group of 6 market vendors in Divisoria run a weekly paluwagan (rotating savings club) worth ₱3,000 each. Every quarter, one vendor disappears with the pot right before their turn to pay in — the last two members lose ₱6,000 each with no recourse because there's no contract, just trust.

## Solution

TipidCircle enforces the paluwagan rules with a Soroban smart contract. Each member contributes USDC each round; when everyone has paid, the pot automatically transfers to that round's designated recipient. No admin holds the money. No one can skip a round without forfeiting participation.

---

## Stellar Features Used

- **USDC transfers** — per-round contributions pulled from member wallets; pot auto-sent to recipient
- **Soroban smart contracts** — round counter, contribution tracking, auto-payout trigger
- **Trustlines** — each member sets up USDC trustline before joining
- **XLM** — base reserve for wallets

---

## Target Users

| | Detail |
|---|---|
| **Members** | Market vendors, sari-sari store owners, BPO coworkers (Metro Manila / Cebu) |
| **Group size** | 3–12 members per circle |
| **Income level** | ₱10K–30K/month, no bank savings account |
| **Pain** | Trust collapse, manual tracking via group chat, no legal enforcement |

---

## Core Feature (MVP)

```
Admin (Alice) deploys circle with 3 members and ₱1,000/round contribution
  → Alice, Bob, Carol each call contribute() for round 0
  → After Carol's contribution, contract auto-calculates pot = ₱3,000
  → Pot transferred instantly to Alice (round 0 recipient)
  → current_round increments to 1 → circle opens for next round
  → After 3 rounds, Carol receives the last pot → status = Complete
```

Demo-able in under 2 minutes (3 wallet transactions on testnet).

---

## Why This Wins

Paluwagan is culturally universal across SEA and the Filipino diaspora (OFW groups in HK, Italy, Qatar). Moving it on-chain with Soroban eliminates the single biggest failure mode — trust — without requiring any banking infrastructure. Stellar's low fees make ₱100 contribution rounds economically viable.

---

## Optional Edge — AI Integration

A Claude-powered group chat bot (Viber / Telegram) automatically reminds members when their contribution window opens, shows the round schedule, and confirms on-chain receipt — bridging the gap between Stellar transactions and the GC-native UX Filipino users already live in.

---

## Prerequisites

- Rust 1.74+
- Soroban CLI 21.x (`cargo install --locked soroban-cli`)
- Stellar Testnet accounts for each member with USDC trustlines

---

## Build

```bash
soroban contract build
# Output: target/wasm32-unknown-unknown/release/tipid_circle.wasm
```

## Test

```bash
cargo test
```

## Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/tipid_circle.wasm \
  --source <ADMIN_SECRET_KEY> \
  --network testnet
```

## Initialise Circle (3-member example)

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET_KEY> \
  --network testnet \
  -- init \
  --token_contract <USDC_TOKEN_CONTRACT_ADDRESS> \
  --admin GALICE123... \
  --members '["GALICE123...", "GBOB456...", "GCAROL789..."]' \
  --contribution_per_member 1000000000
```

## Sample MVP Invocation — Contribute

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ALICE_SECRET_KEY> \
  --network testnet \
  -- contribute \
  --member GALICE123...
```

Repeat for Bob and Carol — after Carol's call, the pot auto-transfers to Alice.

---

## Deployment Reference

Deploy guide: https://github.com/armlynobinguar/Stellar-Bootcamp-2026

Full-stack reference: https://github.com/armlynobinguar/community-treasury

---

## Vision & Purpose

An estimated ₱100B+ circulates annually in informal Filipino paluwagan groups. TipidCircle makes this capital programmable — future extensions include credit scoring based on on-chain repayment history, enabling members to access microloans anchored to their paluwagan track record.

---

## License

MIT