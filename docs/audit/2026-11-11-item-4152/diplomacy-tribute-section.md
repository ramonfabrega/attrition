## 8. Diplomacy: tribute, gifts and counteroffers

### 8.1 Leaves

- `tribute_stamp`, +0x174, `int[8]`, receiver's record, by giver; a frame.
- `gift_stamp`, +0x194, `int[8]`, giver's record, by receiver; a frame.
- `counteroffer`, +0x314, `int[8]`, indexed by the proposer; 0/1.
- `tribute_demanded`, +0x334, `int[8]`, indexed by the target; 0/1.
- `tributes`, +0x498, `int[6]`, one per good; whole resource units.
- `tribute_sent`, +0x860, `int`; whole units, unscaled.
- `tribute_received`, +0x864, `int`; whole units, after the giver's scale.

`frame` is `Game::frame`; every comparison is signed.

### 8.2 The rule

**The deal record.** Each leader holds `dip[8]` (`Diplomacy`, at +0x692c):
`agree` +0, `any_offer` +4, `treaty` +8, `offers[6]` +0xc, `dows[6]` +0x24,
`attacks[8]` +0x3c. `Leader::action_offer (006d1780)` adds an amount to
`A.dip[B].offers[r]` and subtracts it from `B.dip[A].offers[r]`. A positive
slot in `A.dip[B]` means "A gives B". `action_tribute` and
`action_demand_tribute` route through it; none writes a leaf here.
`Leader::init` sets every record to `Diplomacy::clear_all (0047e030)`
(`treaty = -1`).

**Setup writers.** `Leader::init (006e3930)` zeroes `tribute_stamp`,
`gift_stamp`, `counteroffer`, `tribute_demanded` and `tributes` in two loops
that run whatever its tribe argument, after the memset branch. It also calls
`Leader::reset_score (006e37f0)` (also called by `Setup::build_game
(005ac190)`), which zeroes `tribute_sent` and `tribute_received`. With the
header's bulk writers, these are the only resets; none runs in a match.

**`Leader::action_respond (006d03c0)`**, `T` answering `P` with 0
(withdraw), 1 (accept) or 2 (reject), the only `action_*` storing a leaf.

1. **Entry, every code:** `T.tribute_demanded[P] = 0` and
   `T.counteroffer[P] = 0`, before any branch.
2. **Commit** (code 1, `T.dip[P].agree != 1`). For each good both leaders
   can use, `T` must hold `bucket[r] >= max(offers[r], 0) + price[r]`, or the
   accept is refused. Then, for each such good with `o = T.dip[P].offers[r]
   > 0`:

       bucket[r] = max(bucket[r] − o, 0)
       escrow[r] = max(escrow[r] − o, 0)
       tributes[r] += o            (the unclamped o)

   A second pass does the same with `price[r]`, an attack request's price
   in `T.dip[P].attacks[]` (0 unless `T` is human, `leader_flags & 4`), and
   stores `T.dip[P].dows[r] = price[r]`. Then `agree = 1`. The deal executes now if `P` had agreed, or
   if it asks nothing of `P`; otherwise it waits.
3. **Exchange** (any treaty but 2). Each side's `dows[r]` is first credited
   back to its own bucket and zeroed. Then, per good both can use, with
   `gT = max(T.dip[P].offers[r], 0)` and `gP = max(P.dip[T].offers[r], 0)`:

       T.bucket += scale(P, gP);  P.tributes[r] −= gP
       P.tribute_sent += gP;      T.tribute_received += scale(P, gP)
       consider_tribute(T, P, gP, r)
       P.bucket += scale(T, gT);  T.tributes[r] −= gT
       T.tribute_sent += gT;      P.tribute_received += scale(T, gT)
       consider_tribute(P, T, gT, r)
       T.tributes[r] = max(.., 0); P.tributes[r] = max(.., 0)

   Then `Diplomacy::clear_all` runs on both records (no ledger touched).
   **The `dows` residue:** the refund never lowers `tributes`, so a human's
   paid attack price stays in `tributes[r]` until the next `Leader::init`.
   No decision reads it, but the dump and the checksum do, so a port keeps
   it. The second `dows` zeroing hits `P.dip[P].dows[r]` (0 over 0); a port
   may zero `P.dip[T].dows[r]` instead, or nothing.
4. **`scale(G, x)`** is `LeaderData::scale_tribute (006d5240)` on the giver:
   `pct = epoch[2]·commerce_tribute + base_tribute`, floored at 1, where
   `epoch[2]` is the Commerce line. `pct >= 100` returns `x`. Otherwise it
   returns `(x·pct + c)/100`, with `c` 0 for `x < 20`, 50 for `x < 100`,
   else 99.

**`Leader::clear_agree (006d1af0)`** (me, partner `p`), called on every
offer change and by `action_respond`. When `me.dip[p].agree == 1`, per good
`min(tributes[r], dows[r])`, then (if `offers[r] > 0`) `min(tributes[r],
offers[r])`, move from `tributes` to the bucket; `escrow` is not refunded. Then `clear_dows`, and `agree = any_offer =
0`.

**`consider_tribute (006d1360)`** (receiver `r`, giver `g`, `amount` the
gross give before `scale`, `good`), called only by the exchange, after the
receiver's credit:

- If `amount <= 0`, it does nothing.
- `gift_stamp[g][r] = frame`, unconditionally.
- `tribute_stamp[r][g] = frame` if `r` is human, or if all of these hold:
  `amount >= 100·blacken[g]` (+0x20c); `r`'s post-credit `bucket[good] <
  3000`; and `good == WEALTH` or `econ[r][good] & 4`.

**`Leader::process_taunt (006b8cc0)`** (§9 has its gate) writes `gift_stamp`
too. Taunts 1..5 ask for goods 0, 1, 2, 4, 5. Both leaders must have
`type_avail(g,1) == 4`. The ask is refused within 4500 frames of the last
gift, under the lock test next. If the computer's stock is `> 149`, and
`teams_locked == 0` or it is neutral, it sets `gift_stamp[me][from] = frame`.
It then runs `action_clear_all`, `action_offer(from, g, stock/3)` and
`action_respond(me, from, 1)`: a pure gift, executed at once, stamping
again through `consider_tribute`.

**`Leader::diplomacy (006bc950)`** (computer `me`, other `k`) stores only
these two.

- **`counteroffer`.** Only when `k.dip[me].agree == 1`, the evaluation
  builds a shortfall `s` (formula not read here). When `counteroffer[k]` is
  already set, it takes 99 off `s`.
  - If `s > 0`, no good of mine in the deal is positive, and none exceeds my
    stock: `counteroffer[k] = 1`. It asks for the good with the best
    `resource_value` (at least 0x40; never KNOWLEDGE; default WEALTH),
    rounded up to 100, and calls `clear_agree` both ways.
  - `s <= 0`: accept if `any_proposals`, charge `good_deeds[k]` (+50 if
    `weak[k]` and no tribute period), `counteroffer[k] = 0`.
  - Reject: `k.got_diplo_message = frame`, `counteroffer[k] = 0`, then
    `action_respond(me, k, agree == 1 ? 0 : 2)`.
  - At style 0/8/11 (or 7's case), `num_allies == 0`, no victory timer,
    both computers, `aggression[k] == 0`: `counteroffer[k] = 0`.
- **`tribute_demanded`.** The demand arm runs at team style 0/8/11, or
  style 7 with team 8; any other style skips it. It needs peace both ways,
  `aggression[k] != 0`, the rush rules passed, `!is_tribute_period(me, k)`,
  and `frame >= (3·d + ((seed + who) & 7))·300`. Here `d` is `4 + seed%3`
  for a rusher, `8 + seed%6` for a boomer, else `6 + seed%5`. It declares war
  instead if any of these holds: the cleared-agreement flag; `k.blacken >=
  4`; local `-0x30`; `hire_stamp[k] != 0` and `frame − hire_stamp[k] <=
  13500`; or `k.bucket[WEALTH] < D`, with `D = (epoch + 1)·(k.blacken +
  1)·100`, `epoch` read from `me`'s `data_encrypted + 0xe8` xor 0x63187.
  Otherwise it composes a demand (a random start; the non-WEALTH,
  non-KNOWLEDGE goods `me` is short of; `−min(k stock, D)`; a WEALTH ask),
  calls `action_respond(me, k, 1)`, and then sets `tribute_demanded[k] = 1`.
  It stays 1 after `k` pays, until `me`'s own next `action_respond(me, k,
  *)`.

**Readers that change a decision.**

- `is_tribute_period (006d5b90)` reads `tribute_stamp`: over slots `i`
  (not `me`, `leader_flags & 1`), the first that qualifies (`i == p`;
  `is_ally(L[i], p)` after frame 0, equal lobby team `0..3` at frame 0; not
  a team-8 `i` or `p` under style 7) decides: stamp 0 → 0, `frame − stamp <
  3000` → 1. Callers: `diplomacy` ×3, `Army::do_marching (006f3df0)`,
  `Army::find_target (006f69b0)`.
- `gift_stamp`: the taunt refusal, and `diplomacy`'s ally pricing: my
  give to an ally up to `stock/3` (×4 before frame 4500, ×3/2 before 9000,
  under a flag) is free at style 0/8/11, else free unless `gift_stamp[k] !=
  0` and `frame − gift_stamp[k] < 4500`, which charges
  `resource_value·ask/256`.
- The cadence of `diplomacy` reads `counteroffer` and `tribute_demanded`.
  The period `P` starts at 0x800. When `me.dip[k].agree != 1`, it becomes
  0x2000 if `tribute_demanded[k]`, then doubles if `counteroffer[k]`. If
  `agendas[k] & 8 == 0`, it shifts right by 2 (unsigned). The pass fires
  when `(P − 1) & ((k + 8·me)·(P >> 6) + frame) == 0`. A `tribute_demanded`
  also picks the war message.
- `tributes`: `clear_agree`'s caps and the exchange clamps.
  `LeaderData::log_data` dumps it (`6e7515`).
- `tribute_sent`, `tribute_received`: only `AchieveChartPane::set_chart_data
  (007f2c40)`, no sim reader.

### 8.3 In the crate

- **No writer exists**: no `Diplomacy` record, `action_respond`,
  `clear_agree`, `consider_tribute` or `scale_tribute`, and neither tribute
  constant is loaded. The commit debits the ledger's `escrow`
  (`crates/sim/src/economy.rs:687`).
- **Reader stubs:** `tribute()` returns `false` at
  `crates/sim/src/ai/army.rs:3262` and at
  `crates/sim/src/ai/army/target_gate.rs:523` (the `find_target` gate). One
  `is_tribute_period` port replaces both.
- **SEAMs:** `Sim::diplomacy_with` (`crates/sim/src/leaders/diplomacy.rs:177-179`)
  states that `agree` is never 1 and that neither flag is set. The cadence
  at `:180` omits both terms. The style-0/8/11 return at `:470` is where
  the demand arm goes.
- **Checksum.** The leaves are `crates/sim/src/checksum/leaders.rs:65, 66,
  85, 86, 118, 178, 179`, answered by `leader_long`'s `_ => None` (`:1305`).
  Each is an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1774, 1790, 1841-1845`; count
  read from the code, not the verdicts); a port removes them. By 8.4, 0 is
  exact for a style-1 game with no human. `coverage.rs:295` lists `tributes`,
  not `tribute_sent`/`tribute_received`.
- `process_taunt`'s `gift_stamp` waits on §9's chain.

### 8.4 Reached by a plain game

**An all-computer skirmish at team style 1 reaches no event writer**, so
all seven leaves read 0 throughout and the crate's stubs are exact. Command
routes are interface code, the taunt route needs a human's chat,
`ScenarioFuncSet::*` is triggers, and `diplomacy` originates no deal at
style 1 (demand `6be556`; alliance offers `6c09c8`; peace arm `6bdf56`
with `6c0a39`, as every enemy has `aggression[k] = 1`).

The leaves move with a human (a diplomacy-window deal, or a resource taunt
to an allied computer at any style), or at team style 0/8/11, human or
not. The `dows` residue needs a human. No Conquer the World or multiplayer
path is needed.

### 8.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/diplomacy-tribute-*.md`); no diff backs it
yet. Verdicts: 63 confirmed, 4 corrected, 4 open.

Corrections that changed a rule:

- `counteroffer = 1` needs no positive give of mine and none over my stock,
  not a "clear everything" flag.
- The demand's war test and `D` use `k.blacken`, not a counter.
- `log_data` dumps `tributes`.

Confidence is **high** on every store and predicate, each read in the
listing. It is **medium** on `scale_tribute`'s constants.

Witness, **not a pinned diff**: run1086's taunt gift (`docs/PARKED.md`
3470a), where the ally's `gift_stamp[0]` reads the frame and every
`tributes` and `escrow` slot reads 0 at frame end (commit and exchange run
in one call).

### 8.6 Not established

- **The shortfall formula** (decompile `:2676`–`:2777`) is unread, and so is
  the `6c035d` arm (a positive shortfall with a positive give of mine),
  which stores no leaf here.
- **Unnamed:** the demand's local `-0x30`, the epoch word at
  `data_encrypted + 0xe8` (`epoch[0]`?), and the pricing's scaling flag.
  Reading each one's writer settles it.
- **`scale_tribute`**: the load scale of `commerce_tribute` and
  `base_tribute` (`Constants::init`), and its rounding branches, which were
  not re-listed. A deal captured above Commerce 0 pins them.
- **run1086** (the zeros; a predicted human `tribute_stamp[1] = 661`): a
  packet check makes them a diff.
- **`walk_data`** in load mode was not re-read.
