# Continuous Rewards: State & Math Guide

A plain-English guide to how the continuous reward pool works, what state lives where, and how every instruction changes it.

---

## The Two Accounts

There are only **7 fields that matter** across 2 on-chain accounts.

```
┌─────────────────────────────────────────────────────────────────┐
│  REWARD POOL  (one per pool — the "global scoreboard")          │
│                                                                 │
│  reward_per_token   ← the global odometer. goes UP on every    │
│                       DistributeReward. never goes down.        │
│                       "total rewards earned per 1 token,        │
│                        since the pool was created"              │
│                                                                 │
│  opted_in_supply    ← sum of ALL users' last_known_balance.     │
│                       the denominator. "how many tokens are     │
│                       we splitting rewards across?"             │
│                                                                 │
│  total_distributed  ← running total of tokens sent INTO vault   │
│  total_claimed      ← running total of tokens sent OUT of vault │
│                       (these two are just bookkeeping)          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  USER REWARD ACCOUNT  (one per user per pool — "my scorecard")  │
│                                                                 │
│  reward_per_token_paid  ← MY snapshot of the odometer.          │
│                           "last time I settled up, the global   │
│                            odometer was at THIS number"         │
│                                                                 │
│  accrued_rewards        ← tokens I've EARNED but NOT claimed.   │
│                           my piggy bank.                        │
│                                                                 │
│  last_known_balance     ← "the pool THINKS I hold this many    │
│                            tokens." may be stale!               │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Core Trick

The pool maintains one global counter (`reward_per_token`) that only goes up. Each user has a personal snapshot of where that counter was last time they settled. The difference tells you what you earned — no matter how many distributions happened in between.

```
Timeline:
         Distribute     Distribute      Alice claims
         100 tokens     200 tokens
            |               |               |
            v               v               v
Pool RPT:  0 --> 100e12 --> 300e12 -------> 300e12
                                            |
Alice RPT: 0 -----------------------------> 300e12  (snapshotted)
                                            |
                              delta = 300e12 - 0 = 300e12
                              earned = alice_bal * 300e12 / 1e12

She doesn't care that there were 2 distributions.
One subtraction catches her up on EVERYTHING she missed.
```

### The Math

On each `DistributeReward`:
```
reward_per_token += amount * REWARD_PRECISION / opted_in_supply
```

On each `update_user_rewards` (called before any user state change):
```
delta   = pool.reward_per_token - user.reward_per_token_paid
earned  = user.last_known_balance * delta / REWARD_PRECISION
user.accrued_rewards += earned
user.reward_per_token_paid = pool.reward_per_token
```

`REWARD_PRECISION` is 1e12. It's a scaling factor to preserve precision in integer division. Without it, distributing 1 token across 1000 users would truncate to 0.

---

## Instruction State Changes

Every instruction and exactly which fields it touches:

```
                    POOL                          USER
                    reward_   opted_in  total_    rpt_     accrued  last_known
                    per_token supply    dist/clm  paid     rewards  balance
                    --------- -------- ---------- -------- -------- ----------
CreatePool          0         0        0/0        (no user yet)

OptIn               .         +bal     .          =pool    .        =wallet
                    "add me to the denominator, snapshot the odometer"

DistributeReward    ^         .        dist+      .        .        .
                    "crank the odometer up. that's it. no per-user work."

SyncBalance         .         +/-diff  .          =pool    +earned  =wallet
                    "settle, then update what the pool thinks I hold"

SetBalance          .         +/-diff  .          =pool    +earned  =authority
                    "same as sync but authority provides the number"

Claim               .         +/-diff* clm+       =pool    -claimed =wallet*
                    "settle, sync*, pay me out, reduce my piggy bank"

OptOut              .         -bal     clm+       (closed) (closed) (closed)
                    "settle, pay me, remove me from denominator, delete me"

RevokeUser          .         -bal     clm+**     (closed) (closed) (closed)
                    "authority kicks me out. maybe I get paid**, maybe not"

ClosePool           (closed)  .        .          (users stranded if active)

.  = unchanged       ^ = increases only       =pool = set to pool.reward_per_token
*  = only for OnChain balance source pools
** = depends on NonVested (pay out) vs Full (forfeit) revoke mode
```

---

## The Settle-Then-Update Rule

Every instruction that changes a user's balance MUST:

1. **Settle first** (`update_user_rewards`) — calculate what the user earned under their OLD balance
2. **Then update** (`sync_user_balance`) — change the balance going forward

Think of it like a paycheck: your boss pays you for hours worked at $20/hr, THEN tells you your new rate is $25/hr. If he flipped the order, you'd get overpaid for the old period.

This ordering is enforced by convention in every processor, not by the type system.

---

## The Stale Balance Problem

Tokens are NOT locked. They stay in the user's wallet. The pool only knows what it was last told.

```
Alice opts in holding 1000 tokens
   |
   |  Alice sells 900 tokens (pool doesn't know!)
   |     |
   |     |  DistributeReward 100 tokens
   |     |     |
   |     |     |  <- Alice earns based on 1000 (her STALE balance)
   |     |     |     not 100 (her ACTUAL balance)
   |     |     |
   |     |     |  Someone calls SyncBalance for Alice
   |     |     |     |
   |     |     |     |  <- NOW pool sees she has 100
   |     |     |     |     but she already got overpaid
   v     v     v     v
```

There is no on-chain mechanism to force timely syncs. The operator must run off-chain bots (cranks) to call `SyncBalance` for all users periodically, or accept the staleness risk.

### Scalability of Syncing

`SyncBalance` requires 6 accounts per call. You can batch ~10 per transaction.

| Pool size | Txs to sync all | Cost | Wall time |
|-----------|-----------------|------|-----------|
| 100 users | ~10 | ~0.00005 SOL | ~4s |
| 1,000 users | ~100 | ~0.0005 SOL | ~40s |
| 10,000 users | ~1,000 | ~0.005 SOL | ~7 min |
| 100,000 users | ~10,000 | ~0.05 SOL | ~1 hour |

`DistributeReward` itself is O(1), but accurate distribution requires O(n) syncs first.

---

## How It Compares to Direct & Merkle

| | Direct | Merkle | Continuous |
|---|---|---|---|
| Tokens locked? | Yes (escrowed) | Yes (escrowed) | No (observed) |
| Recipients known upfront? | Yes | Yes (in tree) | No (self-select) |
| Distribution cost | O(n) create | O(1) create | O(1) distribute |
| Claim cost | O(1) | O(log n) proof | O(1) |
| Trust model | Trustless (funds locked) | Trustless (funds locked) | Trust authority |
| Balance tracking | Fixed amounts | Fixed amounts | Lazy / authority-set |
| Use case | Airdrops | Large airdrops | Ongoing incentives |

The key difference: Direct/Merkle are **committed** distributions (funds locked upfront, recipients known). Continuous is **uncommitted** (funds flow over time, anyone can join).

---

## Worked Example: 5 Users, 3 Distributions

A full walkthrough showing opt-ins at different times, claims at different rates, and the stale balance problem in action.

```
SETUP: tracked_mint = USDC, reward_mint = BONK
All balances in USDC, all rewards in BONK

Legend:
  bal   = last_known_balance (what pool THINKS user holds)
  snap  = reward_index_snapshot (user's odometer snapshot)
  owed  = accrued_rewards (earned but unclaimed)
  RI    = reward_index (pool's global odometer)
  OIS   = opted_in_supply (total tracked across all users)
```

```
t  | Event              | Pool         | Alice        | Bob          | Carol        | Dave         | Eve
   |                    | RI    OIS    | bal snap owed| bal snap owed| bal snap owed| bal snap owed| bal snap owed
===|====================|==============|==============|==============|==============|==============|=============
 0 | Alice opts in      | 0     1000  |1000  0    0  |  -    -    - |  -    -    - |  -    -    - |  -    -    -
   | (holds 1000 USDC)  |              |              |              |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 1 | Bob opts in        | 0     1500  |1000  0    0  | 500  0    0  |  -    -    - |  -    -    - |  -    -    -
   | (holds 500 USDC)   |              |              |              |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 2 | Carol opts in      | 0     1800  |1000  0    0  | 500  0    0  | 300  0    0  |  -    -    - |  -    -    -
   | (holds 300 USDC)   |              |              |              |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 3 | DISTRIBUTE         | 1.0   1800  |1000  0    0  | 500  0    0  | 300  0    0  |  -    -    - |  -    -    -
   | 1800 BONK          |              |              |              |              |              |
   |                    | RI += 1800/1800 = 1.0       |              |              |              |
   |                    |              |              |              |              |              |
   | What each would    |              | 1000 BONK    |  500 BONK    |  300 BONK    |              |
   | earn if claimed:   |              | (1000x1.0)   | (500x1.0)    | (300x1.0)    |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 4 | Alice claims       | 1.0   1800  |1000  1.0  0  | 500  0    0  | 300  0    0  |  -    -    - |  -    -    -
   |                    |              |              |              |              |              |
   | settle: 1.0-0=1.0 |              | earned 1000  |              |              |              |
   | claim: 1000 BONK   |              | owed->0      |              |              |              |
   |                    |              | snap->1.0    |              |              |              |
   |                    |              |              |              |              |              |
   | ** Alice receives 1000 BONK **   |              |              |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 5 | Dave opts in       | 1.0   2000  |1000  1.0  0  | 500  0    0  | 300  0    0  | 200  1.0  0  |  -    -    -
   | (holds 200 USDC)   |              |              |              |              | snap=1.0     |
   |                    |              |              |              |              | (missed #1)  |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 6 | DISTRIBUTE         | 2.0   2000  |1000  1.0  0  | 500  0    0  | 300  0    0  | 200  1.0  0  |  -    -    -
   | 2000 BONK          |              |              |              |              |              |
   |                    | RI += 2000/2000 = 1.0  (now 2.0 total)    |              |              |
   |                    |              |              |              |              |              |
   | What each would    |              | 1000 BONK    | 1000 BONK    |  600 BONK    |  200 BONK    |
   | earn if claimed:   |              |1000x(2-1)    | 500x(2-0)    | 300x(2-0)    | 200x(2-1)    |
   |                    |              |(only #2)     |(both #1+#2!) |(both #1+#2!) |(only #2)     |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 7 | Bob claims         | 2.0   2000  |1000  1.0  0  | 500  2.0  0  | 300  0    0  | 200  1.0  0  |  -    -    -
   |                    |              |              |              |              |              |
   | settle: 2.0-0=2.0 |              |              | earned 1000  |              |              |
   | claim: 1000 BONK   |              |              | owed->0      |              |              |
   |                    |              |              |              |              |              |
   | ** Bob never claimed #1. Doesn't matter! One subtraction (2.0-0) caught up **  |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 8 | Eve opts in        | 2.0   3000  |1000  1.0  0  | 500  2.0  0  | 300  0    0  | 200  1.0  0  |1000  2.0  0
   | (holds 1000 USDC)  |              |              |              |              |              | snap=2.0
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
 9 | Carol sells 200    | 2.0   3000  |1000  1.0  0  | 500  2.0  0  | 300  0    0  | 200  1.0  0  |1000  2.0  0
   | USDC from wallet   |              |              |              |              |              |
   | (now holds 100)    |              |              |              | POOL STILL   |              |
   |                    |              |              |              | THINKS 300!  |              |
   |                    |              |              |              | nobody synced|              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
10 | DISTRIBUTE         | 3.0   3000  |1000  1.0  0  | 500  2.0  0  | 300  0    0  | 200  1.0  0  |1000  2.0  0
   | 3000 BONK          |              |              |              |              |              |
   |                    | RI += 3000/3000 = 1.0  (now 3.0 total)    |              |              |
   |                    |              |              |              |              |              |
   |                    |              |              |              | UNFAIR!      |              |
   | Carol's share is   |              |              |              | pool uses 300|              |
   | based on 300 USDC  |              |              |              | not 100      |              |
   | but she only has   |              |              |              | overpaid by  |              |
   | 100                |              |              |              | 200 BONK     |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
11 | Someone syncs      | 3.0   2800  |1000  1.0  0  | 500  2.0  0  | 100  3.0 900| 200  1.0  0  |1000  2.0  0
   | Carol's balance    |      -200   |              |              |              |              |
   |                    |              |              |              |              |              |
   | settle FIRST:      |              |              |              |              |              |
   |  delta=3.0-0=3.0   |              |              |              | 300x3.0=900  |              |
   |  earned=900        |              |              |              | owed=900     |              |
   |                    |              |              |              | snap->3.0    |              |
   |                    |              |              |              |              |              |
   | THEN sync:         |              |              |              | bal: 300->100|              |
   |  OIS: 3000-200     |              |              |              |              |              |
   |                    |              |              |              |              |              |
   | Carol owes 900 BONK. Fair would have been 700.   |              |              |              |
   | (300x1 + 300x1 + 100x1 = 700 vs 300x3 = 900)    |              |              |              |
   | Overpaid by 200 due to stale balance.             |              |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
12 | Eve claims         | 3.0   2800  |1000  1.0  0  | 500  2.0  0  | 100  3.0 900| 200  1.0  0  |1000  3.0  0
   |                    |              |              |              |              |              |
   | settle: 3.0-2.0=1.0|              |              |              |              |              |earned 1000
   | claim: 1000 BONK   |              |              |              |              |              |owed->0
   |                    |              |              |              |              |              |
   | ** Eve only earns from #3. Snapshot at opt-in (2.0) skipped #1 and #2 **       |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
13 | Alice claims again | 3.0   2800  |1000  3.0  0  | 500  2.0  0  | 100  3.0 900| 200  1.0  0  |1000  3.0  0
   |                    |              |              |              |              |              |
   | settle: 3.0-1.0=2.0|              |              |              |              |              |
   | earned: 1000x2=2000| earned 2000  |              |              |              |              |
   | claim: 2000 BONK   | owed->0      |              |              |              |              |
   |                    |              |              |              |              |              |
   | ** Alice claimed at t4 (1000) and now (2000) = 3000 total across 3 dists **    |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
14 | DISTRIBUTE         | 4.071 2800  |1000  3.0  0  | 500  2.0  0  | 100  3.0 900| 200  1.0  0  |1000  3.0  0
   | 3000 BONK          |              |              |              |              |              |
   |                    | RI += 3000/2800 = 1.071  (now 4.071 total) |              |              |
   |                    |              |              |              |              |              |
   | ** OIS is 2800 now (after Carol sync), not 3000 **              |              |              |
   | ** Same 3000 BONK, fewer tokens splitting it = bigger per-unit slice **        |              |
   |                    |              |              |              |              |              |
   | What each would    |              | 1071 BONK    | 1071 BONK    |  107 BONK    |  643 BONK    | 1071 BONK
   | earn from this     |              |1000x1.071    | 500x(4.07-2) | 100x1.071    | 200x(4.07-1) |1000x1.071
   | dist if claimed:   |              |              | (incl old!)  |              | (incl old!)  |
   |                    |              |              |              |              |              |
   | ** Carol: only 107 now (100 USDC) vs 300 she would've gotten with stale bal ** |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
15 | Carol claims       | 4.071 2800  |1000  3.0  0  | 500  2.0  0  | 100 4.071  0| 200  1.0  0  |1000  3.0  0
   |                    |              |              |              |              |              |
   | settle first:      |              |              |              |              |              |
   |  delta=4.071-3.0   |              |              |              |              |              |
   |       =1.071       |              |              |              | 100x1.071    |              |
   |  earned=107        |              |              |              | owed: 900+107|              |
   |                    |              |              |              |      =1007   |              |
   |                    |              |              |              | snap->4.071  |              |
   |                    |              |              |              |              |              |
   | claim: 1007 BONK   |              |              |              | owed->0      |              |
   |                    |              |              |              |              |              |
   | ** Carol's owed (900) was sitting there since t11 sync.         |              |              |
   |    This dist added 107 more. She claimed the full 1007. **      |              |              |
---|--------------------|--------------|--------------|--------------|--------------|--------------|-----------
16 | Dave claims        | 4.071 2800  |1000  3.0  0  | 500  2.0  0  | 100 4.071  0| 200 4.071  0 |1000  3.0  0
   |                    |              |              |              |              |              |
   | settle:            |              |              |              |              |              |
   |  delta=4.071-1.0   |              |              |              |              | 200x3.071    |
   |       =3.071       |              |              |              |              | earned=614   |
   | claim: 614 BONK    |              |              |              |              | owed->0      |
   |                    |              |              |              |              |              |
   | ** Dave never claimed before. Gets dists #2, #3, #4 in one shot **             |              |
===|====================|==============|==============|==============|==============|==============|=============

SCOREBOARD                              Alice         Bob           Carol         Dave          Eve
────────────────────────────────────────────────────────────────────────────────────────────────────
Total BONK claimed                       3000          1000         1007          614           1000
Still owed (unclaimed)                      0             0            0            0              0
                                        ─────         ─────        ─────        ─────         ─────
Total earned                             3000          1000         1007          614           1000

Total distributed:  9800 (1800 + 2000 + 3000 + 3000)
Total claimed:      6621
Vault still holds:  3179 (unclaimed from Bob/Alice/Eve for dist #4 + rounding)

** Everyone still in the pool (Alice, Bob, Eve) has unclaimed rewards from dist #4.
   They can claim whenever they want — the owed field will hold it until they do. **
```

### Key Takeaways From the Example

1. **Late joiners are handled correctly**: Dave (t5) and Eve (t8) opt in after distributions and only earn from that point forward. The snapshot mechanism prevents them from claiming retroactive rewards.

2. **Lazy claimers are handled correctly**: Bob (t7) skipped distribution #1 entirely and claimed after #2. One subtraction (`2.0 - 0 = 2.0`) caught him up on both. Dave (t16) skipped three distributions and caught up in one shot. No bookkeeping per distribution needed.

3. **Stale balances break fairness**: Carol (t9-t11) sold 200 USDC but the pool didn't know. She earned 900 BONK instead of the fair 700 for the first 3 distributions. The 200 BONK overpayment came at the expense of the vault's reserves.

4. **The pool's odometer (reward_index) only goes up**: 0 -> 1.0 -> 2.0 -> 3.0 -> 4.071. It never decreases. Each user's snapshot marks where they "got on the bus."

5. **OIS affects the per-unit rate**: Distribution #3 (3000 BONK / 3000 OIS = 1.0 per unit) vs Distribution #4 (3000 BONK / 2800 OIS = 1.071 per unit). Same reward amount, fewer tokens in the pool = bigger slice per token. Carol's sync at t11 reduced OIS, benefiting everyone still in the pool for future distributions.

6. **`owed` is the piggy bank**: Carol earned 900 at t11 (via sync) but didn't claim. At t15, she earned 107 more from dist #4, bringing her total owed to 1007. She then claimed the full amount. The owed field accumulates across multiple settlements until the user decides to withdraw.

---

## Operator Guide

Running a continuous reward pool requires off-chain infrastructure to keep balances accurate. This section covers the practical setup for both balance source modes.

### OnChain Mode: gRPC Crank Pattern

For `OnChain` pools, the operator must sync user balances before distributing. The efficient approach: only sync users whose balances actually changed.

```
┌─────────────────┐     gRPC stream        ┌──────────────────────┐
│  Solana Node     │ ────────────────────→  │  Crank Bot           │
│  (Geyser plugin) │  "token account X     │                      │
│                  │   changed balance"     │  dirty_set: {        │
└─────────────────┘                        │    alice,             │
                                            │    bob,              │
                                            │  }                   │
                                            │                      │
                            on cron/trigger: │  1. batch sync dirty │
                                            │  2. clear dirty_set  │
                                            │  3. DistributeReward │
                                            └──────────┬───────────┘
                                                       │
                                                  txs to Solana
```

**Distribution workflow:**

```
1. Subscribe to all token accounts for the tracked mint via gRPC (Geyser)
2. On balance change → add user to dirty set
3. When ready to distribute:
   a. Batch SyncBalance for everyone in dirty set (~10 per tx)
   b. Wait for all sync txs to confirm
   c. Call DistributeReward
   d. Clear dirty set
```

**Cost scaling** (only syncing users with balance changes):

| Users w/ balance changes | Txs needed | Cost | Time |
|--------------------------|------------|------|------|
| 10 | ~1 | ~0.000005 SOL | ~0.4s |
| 50 | ~5 | ~0.000025 SOL | ~2s |
| 500 | ~50 | ~0.00025 SOL | ~20s |
| 5,000 | ~500 | ~0.0025 SOL | ~3 min |

Much cheaper than syncing the entire pool — but requires reliable change detection.

**Gotchas:**

1. **Race condition**: a user moves tokens between "sync batch sent" and "distribute tx sent." The sync lands with the old balance, then the user transfers, then distribute runs with a stale balance. Mitigate by keeping the window tight (send distribute immediately after last sync confirms) or accepting a small staleness window.

2. **gRPC gaps**: if the stream disconnects or the bot restarts, balance changes are missed. Run a periodic full-sweep fallback (sync all users, e.g. once per day) as a safety net.

3. **Batch ordering**: all sync txs must confirm before the distribute tx. Use sequential sends with confirmation, or preflight simulation.

4. **Operator pays**: sync tx fees come out of the operator's wallet, not the users'. This is the operating cost of running a continuous pool.

### AuthoritySet Mode: Off-Chain Balance Management

For `AuthoritySet` pools, the operator controls all balances directly. No gRPC subscription needed — the authority computes balances from their own data source and batch-sets them before distributing.

```
┌─────────────────┐                        ┌──────────────────────┐
│  Data Source     │ ────────────────────→  │  Authority Service   │
│  (database,      │  balances per user     │                      │
│   indexer,       │                        │  1. compute balances │
│   API, etc.)     │                        │  2. batch SetBalance │
└─────────────────┘                        │  3. DistributeReward │
                                            └──────────┬───────────┘
                                                       │
                                                  txs to Solana
```

**When to prefer AuthoritySet over OnChain:**

- Balances come from off-chain sources (points systems, cross-chain holdings, API data)
- Pool has many users (>1,000) and sync costs are a concern
- Operator wants full control over balance timing
- Simpler infrastructure (no gRPC, no change detection)

**Tradeoff:** users must trust the authority to set balances correctly. There is no on-chain verification that the authority's numbers match reality.

### Choosing a Mode

```
Do balances come from on-chain token accounts?
  │
  ├── YES → How many users?
  │           │
  │           ├── < 1,000  → OnChain mode (manageable sync cost)
  │           │
  │           └── > 1,000  → Consider:
  │                          - OnChain mode + gRPC crank (delta-only sync)
  │                          - Periodic Merkle distributions instead
  │                          - AuthoritySet mode w/ indexer snapshots
  │
  └── NO (off-chain data, points, cross-chain) → AuthoritySet mode
```
