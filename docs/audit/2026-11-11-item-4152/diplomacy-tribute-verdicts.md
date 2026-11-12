# Item 4152, family 8 (`diplomacy-tribute`): adjudication

- Slug: `diplomacy-tribute`. Leaves: `tribute_stamp` (+0x174 int[8]),
  `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]),
  `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]),
  `tribute_sent` (+0x860), `tribute_received` (+0x864).
- Readings: `diplomacy-tribute-reading.md` (first), `diplomacy-tribute-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text` listing
  (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of the original.
- Spawn questions: (A) does an all-computer skirmish at the default team
  style reach any writer; (B) `tributes` not reduced for refunded `dows`;
  (C) the zeroed `dows` slot being `P.dip[P]`. Settled in the shared section
  below, then leaf by leaf (appended as settled).

## Shared: the three spawn questions

### (A) Does an all-computer skirmish at team style 1 reach any writer?

The second reading says no (each leaf, section 6); the first answers a
different question (any plain game, a human included) and says yes. Settled
by walking every route into the writers.

1. **Command routes are human-only.** Every `CommandPackage::process_*`
   that reaches `action_offer (006d1780)` or `action_respond (006d03c0)` is
   fed by a `CommandManager::issue_*` (`issue_treaty (00942100)`,
   `issue_accept (00942500)`, `issue_reject (00942600)`, `issue_tribute
   (00942700)`, `issue_demand_tribute (00942810)`, `issue_propose_attack
   (00942920)`), and the only callers of those are `IFaceDiploNeg::*`, the
   `LeaderOut::issue_*` wrappers and `Options::exec (007188c0)` (hotkeys):
   interface code (call-site scan of the listing).
2. **The taunt route is human-only.** `process_taunt (006b8cc0)` acts on
   `incoming_taunt*`, which only `Leader::receive_taunt (006b8bf0)` fills,
   whose only caller is `CommandPackage::process_chat (009454f0)`; the chat
   command is issued by `CommandManager::issue_chat (00942b10)`, called from
   `ChatBox::on_modal_end (0093d560)` and `LeaderOut::issue_chat (006d2c50)`,
   itself called from `IFaceDiploBase::on_button_clicked (008014e0)`,
   `IFaceDiploBase::on_key_click (008017f0)` and `ChatBox::on_modal_end`
   (listing `801722`, `80197c`, `93d6f6`). No sim code sends a chat.
3. **`ScenarioFuncSet::*`** (`offer_tribute (009fc030)`, `accept_offer
   (009fc410)`, ...) is the trigger layer, absent from a random-map skirmish.
4. **`Leader::diplomacy (006bc950)` never originates a deal at team style 1.**
   Its 22 `action_respond` and 4 `action_offer` call sites split into
   (a) answers, every one under `k.dip[me].agree == 1`, `me.dip[k].agree ==
   1` or `any_proposals(me, k)` (`LeaderData::any_proposals (006d5ad0)` reads
   `me.dip[k]`: `treaty != -1` or a nonzero offer or attack, all written only
   by someone's `action_*`; `Leader::init` sets every record to
   `Diplomacy::clear_all (0047e030)`'s `treaty = -1`), and (b) four
   originators, each gated off at style 1:
   - the demand (`6be85d` arm): `6be54c`–`6be556`, team style not 0/8/11
     (and not 7's team-8 case) jumps to `LAB_006bed3f`;
   - the alliance-with-attack offer (decompile `:2310`–`:2314`): `6c09b6`–
     `6c09c8`, team style not 0/8/11 jumps to `6c0a2e`;
   - the peace offer (`LAB_006c0f50`, `:2320`–`:2452`) and everything past
     `6c0a2e`: **`6c0a2e cmpl $0, 0x210(%edi,%esi,4)` / `6c0a39 jne
     6be98d`** — `aggression[k] != 0` leaves the target. At style 1
     `LAB_006bdf25` stores `aggression[k] = 1` for every target that is not
     me or a mutual ally (`6bdf4d`–`6bdf56`), so an enemy never passes; a
     mutual ally (stored 0, `6bdf78`) passes but then fails the
     alliance arm's "at peace, not allied" test and `LAB_006c13ad`'s
     "not allied" test (`:2494`–`:2504`), neither of which offers anything;
   - the gang-up alliance offer to a top-scoring human (`:2600`–`:2635`):
     reached only when the `:2128`–`:2150` disjunction is false, and its
     last disjunct (team style not 0/8/11) is true at style 1 (`6c09c8`).
   So at style 1 with no human, no `Diplomacy` record ever leaves
   `clear_all`, `k.dip[me].agree` stays 0, and **no `action_respond`,
   `consider_tribute` or `clear_agree` call is ever made**.

**Verdict: confirmed (the second reading), with one refinement.** An
all-computer skirmish at team style 1 reaches no *event* writer of any of the
seven leaves. It does reach their *setup* writers: `Leader::init
(006e3930)`'s unconditional zeroing loops (`6e3c21`, `6e3c27`, `6e3d75`,
`6e3d7b`, `6e3e8b`; below) and `Leader::reset_score (006e37f0)` (`6e3895`,
`6e389b`), which store 0 over the 0 that `Leaders::clear` or `Leader::init`'s
`memset` already left. Every leaf therefore reads 0 for the whole of such a
game, and the crate's stubs (`ai/army.rs:3262`, `ai/army/target_gate.rs:523`
returning `false` for `is_tribute_period`) equal the original exactly there.
The deciding lines are `6bdf56` with `6c0a39` (the peace arm), `6c09c8` (the
two alliance arms) and `6be556` (the demand).

### (B) `tributes` not reduced for the refunded `dows`

Both readings raise it; the first calls it a reading only. **Confirmed.**
At accept, the attack-price pass (decompile `:185`–`:204`) lowers T's
bucket and `escrow` by `price[r]` (each clamped at 0), raises
`T.tributes[r]` by `price[r]` (`6d0857`) and stores `T.dip[P].dows[r] =
price[r]` (`6d0863`). At execution the exchange credits each side's `dows`
back to its own bucket (`6d0e57` for T, `6d0e6f` for P) and zeroes the slot,
then subtracts only `max(offers[r], 0)` from `tributes` (`6d0f17`,
`6d0f7e`); the clamps (`6d0fd4`, `6d0fe2`) only lift a negative value. The
tail's two `Diplomacy::clear_all` (`6d12c1` on `T.dip[P]`, `6d12d5` on
`P.dip[T]`) zero the records and touch no ledger, and they reset `agree`, so
no later `clear_agree` for that pair can take the residue back. No other
code charges the price: the tail's attack loop (decompile `:520`–`:541`)
stamps `hire_stamp`/`hire_who` and calls `action_declare`, with no
`bucket_add`. **The rule**: an executed deal in which a *human* side had an
attack request priced (`price` is multiplied by `leader_flags & 4 != 0`,
decompile `:110`) refunds that price in full to the human's bucket and leaves
`tributes[r]` permanently higher by it, until `Leader::init`. Behaviourally
inert (the residue only loosens `clear_agree`'s `min` cap, which an
un-inflated ledger never reaches either), but `tributes` is in the `LEADERS`
dump (below, leaf 5) and the checksum walk (`checksum/leaders.rs:118`), so a
port must keep the residue. A human is required, so no all-computer game
shows it.

### (C) The zeroed `dows` slot is P's own

Both readings raise it. **Confirmed as a listing fact, and inert.** The
exchange's second zeroing is `6d0e8e andl $0x0,(%eax)` through `-0x30(%ebp)`
= `0xe40ce0 + P·0x6f48 + 4r` (`6d0e23`–`6d0e2e`, advanced at `6d0fe9`);
`0x6f48 = 0x6eec + 0x5c`, so the word is `leaders[P].dip[P].dows[r]`. The
credit two instructions before (`6d0e5c`–`6d0e6f`) reads the right word,
`P.dip[T].dows[r]` (`0xe40ce0 + 4·(T·0x17 + P·0x1bbb + r)`), so P is paid
once, and the stale `P.dip[T].dows[r]` is zeroed by the tail's
`clear_all(P.dip[T])` at `6d12d5` before anything reads it (nothing in the
loop or the tail does). The self record `P.dip[P]` is never written nonzero
by any route (no `action_*` targets its own slot), so the stray store
writes 0 over 0. **A port writes `P.dip[T].dows[r] = 0` or nothing; both
match the original.**

## Shared: the setup writers, settled once

- **`Leader::init (006e3930)`.** One-sided on each side. The second reading
  gives only the `memset(this, 0, 0x6929)` under `param_2 < 0`
  (`6e3948 jns 6e3a36` skips it, `6e398c`–`6e3995`) and discards the
  `this+0xd4`-based stores as other fields. The first gives the per-slot
  zeroing loop and calls it unconditional. **Both confirmed, and the first's
  loop is the one that matters**: the loop `6e3c0d`–`6e3d93` (decompile
  `:180`–`:200`) sits after the `param_2 < 0` block closes (`:144`) and
  stores 0 at `0xa0(%esi)` = +0x174 (`6e3c21`), `0xc0(%esi)` = +0x194
  (`6e3c27`), `0x240(%esi)` = +0x314 (`6e3d75`), `0x260(%esi)` = +0x334
  (`6e3d7b`), `esi = this + 0xd4 + 4j`; the goods loop `6e3e62`–`6e3f44`
  (decompile `:279`–`:300`, top level) stores 0 at `0x18(%edx)` = +0x498
  (`6e3e8b`, `edx = this + 0x480 + 4r`). So `Setup::build_game`'s
  non-negative `param_2` path, which skips the `memset`, still zeroes five
  of the seven leaves; `reset_score` (called at `:140`, unconditionally)
  zeroes the other two. The second reading's discards at `0x17c(%esi)` and
  `0x19c(%esi)` (`weak`, `dow`) are right; it missed the smaller
  displacements.
- **`Leaders::clear (006ed5d0)`** memsets `[0, 0x6929)` of every slot:
  agree. **`LeaderData::walk_data (006d6750)`** (second only) is the save
  game's raw copy: not a plain-game writer; not re-read.

## Leaf 1: `tribute_stamp` (+0x174, int[8])

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | writer `consider_tribute (006d1360)` `6d13d8`, `R.tribute_stamp[G] = frame` | yes | yes | agree |
| 2 | called only from `action_respond`'s exchange (`6d0f4f`, `6d0fbb`), receiver `this`, after the receiver's `bucket_add` | yes | yes | agree |
| 3 | predicate: `amount > 0`; then R human, or (`amount >= 100·G.blacken` and R's post-credit `bucket[g] < 3000` and (`g == 2` or `R.econ[g] & 4`)) | yes | yes | agree |
| 4 | `amount` is the giver's gross (pre-scale) amount | implied | yes | agree |
| 5 | reset: `Leader::init`'s loop (`6e3c21`, unconditional) | yes | no (memset only) | one-sided |
| 6 | reader `is_tribute_period (006d5b90)`: first qualifying slot decides; stamp 0 returns 0 at once (`6d5c8e`) | yes | yes | agree |
| 7 | callers of the reader: `diplomacy` ×3, `Army::do_marching (006f3df0)`, `Army::find_target (006f69b0)` | yes | yes | agree |
| 8 | crate: no writer; reader stub `ai/army.rs:3262` | yes | yes | agree |
| 9 | crate: second stub `ai/army/target_gate.rs:523` | no | yes | one-sided |
| 10 | plain game: any accepted gift; human receiver unconditional | yes | yes (human or style 0/8/11 demand) | agree |
| 11 | all-computer style 1 reaches it | — | no | settled (A) |

### Settled

- **Predicate (3), spot-checked.** Listing `6d1360`–`6d13e1`: `6d1367 jle`
  (signed `amount <= 0` returns); `6d138b testb $4,(%edx)` (R human →
  store); `6d1396 imull $0x64, 0xe3a59c(%eax)` with `eax = G·0x6eec` (the
  giver's `blacken`, +0x20c) and `6d13a1 jl` (signed); `6d13ae`–`6d13c1`
  R's `data_encrypted->bucket[g] ^ 0x8221 >= 0xbb8` skips (signed);
  `6d13c3 cmpl $2` / `6d13c8 testb $4, 0x450(%edx,%ecx,4)` (R's `econ`).
  **Confirmed.** The frame is `*(0xc061ec)+0x550`, `Game::frame`.
- **Reader (6), spot-checked.** `6d5bc0`–`6d5c9d`: skips `i == this.who`
  and `!(leader_flags & 1)`; `i == p` qualifies (`6d5bd7`); team style 7
  disqualifies a team-8 `i` or `p` (`6d5bf9`, `6d5c2a` → `6d5cad`); at
  frame 0 lobby team `0..3` equal (`6d5c5f js`, `6d5c64 jge 4`,
  `6d5c85`); after frame 0 `is_ally(L[i], p)` (`6d5c40`); stamp 0 →
  `6d5cd0` (return 0); `frame − stamp < 0xbb8` → return 1. **Confirmed.**
- **Reset (5): confirmed (first)**, deciding line `6e3c21`.
- **Crate (9): confirmed (second).** `ai/army/target_gate.rs:523`, `fn
  tribute(&mut self, _b: usize) -> bool { false }`, is a second home of the
  same read (the `find_target` gate); the first reading missed it.

### Rule to port

- **Writer**: in a `consider_tribute(r, g, amount, good)` called twice per
  good from the exchange of an `action_respond` port, after the receiver's
  bucket credit: if `amount > 0` and (`r` human, or `amount >=
  100·blacken[g]` and `bucket[r][good] < 3000` and (`good == WEALTH` or
  `econ[r][good] & 4 != 0`)), `tribute_stamp[r][g] = frame`. Integers;
  `amount` the gross give, before `scale_tribute`.
- **Reset**: 0 at leader setup (every `Leader::init`).
- **Reader**: `is_tribute_period(me, p)` as above, the 3000-frame window,
  first qualifying slot decides; it replaces both crate stubs.
- **Crate site**: none yet; a new `consider_tribute` beside an
  `action_respond` port (no `leaders/tribute.rs` exists).
- **Plain game**: needs a human (taunt gift, diplomacy-window deal) or team
  style 0/8/11; an all-computer style-1 game holds 0 throughout, and the
  stubs are exact there.
- **Confidence**: high, listing-checked end to end. Open: nothing on the
  writer; the run1086 human `tribute_stamp[1] = 661` prediction is not
  checked against a dump.

## Leaf 2: `gift_stamp` (+0x194, int[8])

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | `consider_tribute` `6d1384`: `G.gift_stamp[R] = frame` when `amount > 0`, before the human/blacken test | yes | yes | agree |
| 2 | `process_taunt (006b8cc0)` `6b8ecd`: `this.gift_stamp[from] = frame` only when `teams_locked == 0` or `this` neutral, stock `> 149` | yes | yes | agree |
| 3 | `process_taunt` predicate: `this` computer, active, `from != who`, mutual allies; taunt 1..5 → goods 0,1,2,4,5; both `type_avail(g,1) == 4`; 4500-frame refusal under the same lock test | yes | yes | agree |
| 4 | the grant then `action_clear_all`, `action_offer(from, g, stock/3)`, `action_respond(this, from, 1)`, a pure gift executed at once, re-stamping via `consider_tribute` | yes | yes | agree |
| 5 | reset: `Leader::init` loop `6e3c27` | yes | no | one-sided |
| 6 | reader `process_taunt` (refusal) | yes | yes | agree |
| 7 | reader `Leader::diplomacy` `6bf61e`: effect | not followed | an ally's ask up to a third of my stock is free unless (style not 0/8/11) I gifted that ally within 4500 frames, then charged `resource_value·ask/256` | one-sided |
| 8 | crate: no writer, no reader; checksum `checksum/leaders.rs:66` | yes | yes | agree |
| 9 | plain game: taunt needs a human allied to a computer, any team style | yes | yes | agree |
| 10 | all-computer style 1 | — | no | settled (A) |

### Settled

- **Writer (1), spot-checked.** `6d1384 movl %eax, 0xe3a524(,%ecx,4)`,
  `ecx = param_1·0x1bbb + this.who` (`6d136e`–`6d137b`): giver's record,
  receiver's index, after only the `6d1367` amount test. **Confirmed.**
- **Reset (5): confirmed (first)**, `6e3c27`.
- **Reader (7): confirmed (second).** Decompile `:1753`–`:1837`, listing
  around `6bf61e`: the ask is `me.dip[k].offers[r]` (`0x6938(%esi)`, my
  give), scaled ×4 before frame 4500 and ×3/2 before 9000 when `*local_44`
  is 0; if it is `< 1`, or `k` is not my ally, or it exceeds `stock/3`, it
  is charged; otherwise it is free at team style 0/8/11 and, at any other
  style, free unless `gift_stamp[k] != 0 && frame − gift_stamp[k] < 4500`
  (`LAB_006bf618` → `LAB_006bf63d`). The style-7 test nested inside the
  0/8/11 branch can never hold, so styles 0/8/11 are free outright.
- **Reachability (9, 10).** The taunt path is human-only (A, step 2), so
  in a style-1 game with a human ally the computer's grant stamps twice in
  one frame (when unlocked) or once (locked, through `consider_tribute`).

### Rule to port

- **Writers**: `consider_tribute`: `amount > 0` → `gift_stamp[g][r] =
  frame`. `process_taunt`: on a granted resource taunt, when `teams_locked
  == 0` or the computer is neutral, `gift_stamp[me][from] = frame` before
  the gift.
- **Reset**: leader setup.
- **Readers**: the taunt refusal (4500 frames, same lock test) and the
  proposal pricing above.
- **Crate site**: none; a `process_taunt` port (it needs the chat-command
  taunt queue) and the `consider_tribute` above.
- **Plain game**: only with a human; 0 throughout an all-computer style-1
  game.
- **Confidence**: high. Open: the pricing loop's `*local_44` (a flag of
  `Leader::diplomacy`) not named.

## Leaf 3: `counteroffer` (+0x314, int[8])

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | writers `Leader::diplomacy` `6c005e` (=1), `6bf3ea`, `6c0840`, `6c086b` (=0); `action_respond` `6d03e0` (=0 on entry, every code) | yes | yes | agree |
| 2 | writer `Leader::init` loop `6e3d75` | yes | no | one-sided |
| 3 | `−99` forgiveness on the shortfall when the slot is set (`6bfe1c`–`6bfe2a`) | yes | yes | agree |
| 4 | `=1` predicate | shortfall `> 0`, `local_54 == 0` (glossed as the "clear everything" flag), `local_48 == 0` (not followed) | shortfall `> 0` and `-0x44` ("already asked") `== 0` | disagree |
| 5 | `=1` effect: best `resource_value` good (≥ 0x40, never KNOWLEDGE, default WEALTH), `clear_agree` both ways, ask rounded up to 100 | yes | yes | agree |
| 6 | `=0` at `6c086b`: shortfall `<= 0` → accept if `any_proposals`, charge `good_deeds[k]` (`+50` when `weak[k]` and not a tribute period) | yes | yes | agree |
| 7 | `=0` at `6c0840`: the reject arm; `k.got_diplo_message = frame`; `action_respond(me,k, agree==1 ? 0 : 2)` | partly (no `got_diplo_message`) | yes | one-sided |
| 8 | `=0` at `6bf3ea`: style 0/8/11 (or 7's case), `num_allies == 0`, no victory timer, both computers, `aggression[k] == 0` | yes | "neither human and `-0x28` is 0" | agree (second's `-0x28` is the pointer to `aggression[k]`, `6be4ab`–`6be4b1`) |
| 9 | reader: the cadence doubling while `dip[k].agree != 1` | yes | yes | agree |
| 10 | crate: SEAM `leaders/diplomacy.rs:177`–`:179`; cadence omits it | yes | yes | agree |
| 11 | plain game: a proposal to a computer (human, or a computer's own under style 0/8/11) | yes | yes | agree |

### Settled

- **(4) corrected: both glosses are off.** The pricing walk at
  `LAB_006bf4b6` reuses both slots: `6bf4b8 movl $0,-0x44(%ebp)`, `6bf4bf
  movl %eax,-0x50(%ebp)` (0), then per good `6bf527`–`6bf52c` sets `-0x50 =
  1` when `me.dip[k].offers[r] > my stock[r]`, and `6bf531`–`6bf533` sets
  `-0x44 = 1` when that offer is positive. Every path to the counteroffer
  site passes `LAB_006bf4b6` (the treaty-2 pricing ends `goto
  LAB_006bf4b6`, decompile `:2040`), so the first reading's "clear
  everything" meaning (the slot's earlier use, `6be4a8`) is dead by then.
  At the site: `6bfe30 testl -0x50; je 6bffef`; `6bffef`–`6bfffc`: shortfall
  `<= 0` → `6c0865` (accept), else `-0x44 != 0` → `6c035d`, else fall into
  the `=1` arm. **Rule: `counteroffer[k] = 1` iff the shortfall (after the
  −99) is positive, no good of mine in the deal is positive (`-0x44 == 0`),
  and none exceeds my stock (`-0x50 == 0`)** — the computer counters a deal
  that asks it for nothing yet still falls short, by demanding goods. The
  second reading names one of the two conditions; the first names both but
  mislabels them. Deciding lines `6bfe32` and `6bfffc`.
- **(7) confirmed (second).** `6c0832`–`6c0838`: `0xe3a680(%ecx)`, `ecx =
  k·0x6eec` → `k.got_diplo_message` (+0x2f0) = frame, before the `6c0840`
  store.
- **(2) confirmed (first)**, `6e3d75`.
- **(9) spot-checked.** `6bcaba`–`6bcaf0`: base 0x800; when
  `me.dip[k].agree != 1`, `tribute_demanded[k] != 0` → `-0xa4` = 0x2000
  (`6bca20`), then `counteroffer[k] != 0` doubles; `agendas[k] & 8 == 0`
  shifts right 2 (unsigned). Fires when `(P − 1) & ((k + 8·me)·(P >> 6) +
  frame) == 0` (`6bcaf5`–`6bcb0e`). **Confirmed.**

### Rule to port

- **Writers**: `Leader::diplomacy`'s proposal evaluation (computer `me`,
  proposer `k`, reached only when `k.dip[me].agree == 1`): `=1` as above;
  `=0` on its accept (`6c086b`), reject (`6c0840`) and the style-0/8/11
  no-allies acceptance (`6bf3ea`); and `=0` on entry to every
  `action_respond(me, k, *)`.
- **Reset**: leader setup; the entry clear.
- **Readers**: the cadence (×2) and the −99.
- **Crate site**: `Sim::diplomacy_with`'s `LAB_006bf0e0` SEAM and an
  `action_respond` port; the cadence at `leaders/diplomacy.rs:180` then reads
  it.
- **Plain game**: needs a proposal to a computer: a human's, or a
  computer's under style 0/8/11. 0 throughout an all-computer style-1 game
  (A).
- **Confidence**: high on stores and predicates now listing-read; the
  shortfall formula itself (decompile `:2676`–`:2777`) is not this leaf's
  and stays unread. Open: what `6c035d` (positive shortfall with a positive
  give of mine) does; it stores no leaf of this family.

## Leaf 4: `tribute_demanded` (+0x334, int[8])

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | writers `Leader::diplomacy` `6be85d` (=1, after its own `action_respond(me,k,1)`), `action_respond` `6d03d8` (=0 on entry) | yes | yes | agree |
| 2 | writer `Leader::init` loop `6e3d7b` | yes | no | one-sided |
| 3 | gate: team style 0/8/11 (or 7's team-8 case); style 1 → `LAB_006bed3f` | yes | yes | agree |
| 4 | skip when `is_tribute_period(me,k)` or `frame < floor` (`-0x3c`), floor `(3·span + ((seed + who) & 7))·300` | yes | yes | agree |
| 5 | war instead when: the cleared-agreement flag (`-0x50`) | yes | yes | agree |
| 6 | ... "a counter above 3" (first) vs `k.blacken >= 4` (second) | counter | `k.blacken` | disagree |
| 7 | ... `-0x30` set | yes (`local_34`) | yes | agree |
| 8 | ... `hire_stamp[k]` set and young: "under 13501 (0x34bd)" vs "younger than 13501 (0x34bc)" | yes | yes | agree in substance |
| 9 | ... `k`'s WEALTH `< D`, `D = (epoch + 1)·(X + 1)·100`, X "count" (first) / `k.blacken` (second) | count | `k.blacken` | disagree |
| 10 | demand composition (random start, non-WEALTH non-KNOWLEDGE goods short for me, `−min(k stock, D)`, WEALTH ask) | yes | yes | agree |
| 11 | the computer's slot stays 1 after the target pays; only its own `action_respond(me,k,*)` clears it | yes | yes | agree |
| 12 | readers: cadence 0x2000, the war chat table (`6be9f5`) | yes | yes | agree |
| 13 | crate: SEAM `leaders/diplomacy.rs:177`–`:179`; the style-0/8/11 SEAM at `:470` | yes | yes | agree |
| 14 | plain game: style 0/8/11 only; reachable with no human at style 0 | yes | yes | agree |

### Settled

- **(6) and (9) corrected for the first, confirmed for the second.**
  `6be664`–`6be670`: `-0x18(%ebp)` is `k·0x6eec` (it also indexes `k`'s
  `data_encrypted` at `6be6b4`), `movl 0xe3a59c(%ecx),%edx` is `k.blacken`
  (+0x20c), `cmpl $4; jge` → war. `6be6a4 leal 0x1(%edx)` reuses it: `D =
  ((me.data_encrypted+0xe8) ^ 0x63187) + 1) · (k.blacken + 1) · 100`
  (`6be698`–`6be6ba`), compared signed against `k.bucket[WEALTH]`
  (`6be6bd`–`6be6ca`, `jl` → war) and stored into `-0x3c` for the demand
  caps. No counter is involved.
- **(8)**: `6be68b`–`6be692`: `frame − hire_stamp[k] <= 0x34bc` → war (when
  the stamp is nonzero), i.e. under 13501 frames. Both readings state this.
- **(3), (4), (5), (7) spot-checked**: `6be54c`–`6be556` (style),
  `6be639`–`6be640` (`is_tribute_period`), `6be651`–`6be654` (floor),
  `6be65a`–`6be65e` (`-0x50`), `6be676`–`6be67a` (`-0x30`). **Confirmed.**
- **(2) confirmed (first)**, `6e3d7b`.

### Rule to port

- **Writers**: `=1` at the end of the demand arm of `Leader::diplomacy`
  (computer `me`, target `k`, style 0/8/11 or 7's case, at peace both ways,
  `aggression[k] != 0`, past rush rules, `!is_tribute_period(me,k)`,
  `frame >= floor`, and none of: cleared agreement, `k.blacken >= 4`,
  `-0x30`, `hire_stamp[k]` within 13500 frames, `k.WEALTH < D`); `=0` on
  entry to every `action_respond(me, k, *)`.
- **Reset**: leader setup; the entry clear (the demander's own call only).
- **Readers**: cadence (0x2000 base) and the war message table.
- **Crate site**: `Sim::diplomacy_with` past its style-0/8/11 SEAM
  (`leaders/diplomacy.rs:470`); the cadence term at `:180`.
- **Plain game**: style 0/8/11 only, with or without a human; 0 throughout
  any style-1 game.
- **Confidence**: high. Open: `-0x30` (`local_34`) is not named by either
  reading; the epoch word at `data_encrypted+0xe8` is called `epoch[0]` by
  the first and left unindexed by the second (not settled here).

## Leaf 5: `tributes` (+0x498, int[6])

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | `action_respond` commit: `6d07b5` (`+offer`, goods both have, after the affordability pass) and `6d0857` (`+price`, human T only, also `dows[r] = price`) | yes | yes | agree |
| 2 | commit adds the full amount though the bucket and `escrow` debits clamp at 0 | implied | yes | agree |
| 3 | exchange: `6d0f17` (`P.tributes[r] −= g_P` through the `6d0e19` pointer), `6d0f7e` (`T.tributes[r] −= g_T`), clamps `6d0fd4`, `6d0fe2` | yes | yes | agree |
| 4 | `clear_agree (006d1af0)` refund: when `agree == 1`, per good `min(tributes, dows)` then (offer `> 0`) `min(tributes, offers)` back to the bucket | yes | yes | agree |
| 5 | `Leader::init` loop `6e3e8b` | yes | no | one-sided |
| 6 | the `dows` refund leaves the price in `tributes` | raised, open | raised | settled (B): confirmed |
| 7 | the second `dows` zeroing hits `P.dip[P]` | raised, open | raised | settled (C): confirmed, inert |
| 8 | readers: `clear_agree`, the clamps, and `log_data` (`6e78ed`) | yes | no dump reader ("`log_data` hits are other bases") | disagree |
| 9 | a one-way gift goes `+x`, `−x` in one call; end-of-frame dumps read 0 (run1086) | yes | — | one-sided |
| 10 | crate: none; neighbour `economy.rs:687` `escrow`; checksum `:118` | yes | yes | agree |
| 11 | plain game: any accepted deal with goods; lasting value only while waiting on the other side | yes | yes (human or style 0/8/11) | agree |

### Settled

- **(8) corrected for both.** `LeaderData::log_data (006e5110)`'s per-good
  loop sets `ebx = this + 0x480` (`6e73d4`), walks 6 goods (`6e7910 leal
  0x4(%ebx)`, `6e7916 jne`), and pushes `-0x18(%ebx)` (`escrow`), `(%ebx)`
  (`escrow_rate`), **`0x18(%ebx)` = `tributes[r]` at `6e7515`**, `-0x30`
  (`econ`), `0x30` (`base_rate`), ... and `0x498(%ebx)` = +0x918
  (`bonus_cap`) at `6e78ed`. So the dump does carry `tributes` (the second
  reading's scan, keyed on displacement 0x498, could not see it), and the
  first reading's address is the wrong field's.
- **(1), (2), (4) spot-checked.** `6d076c`–`6d07b5`: bucket `max(b − o,
  0)` (`cmovs`), `escrow` (`0xe3a7f8` = +0x468) `max(e − o, 0)`, then
  `addl %esi, 0x498(%ebx,%edx,4)` with the unclamped `o`. `6d1af0`–`6d1b91`:
  `agree != 1` skips the refunds; `cmpl $0, 0x18(%edi)` is `dows`
  (+0x6950), `min` by `cmovl` (signed), `bucket_add`; then offers `> 0`
  (`6d1b44 jle`) likewise; then `clear_dows`, and `agree` and `any_offer`
  are zeroed on both paths (`6d1b84`, `6d1b87`). `escrow` is not refunded.
  **Confirmed.**
- **(5) confirmed (first)**, `6e3e8b`.
- **(9)** is consistent with the listing (the pure-gift path escrows and
  exchanges inside one call); the run1086 packet check named in the first
  reading would make it a diff. Not re-checked here.

### Rule to port

- **Writers** (units: whole resource units, the bucket's own integers):
  on `action_respond(T, P, 1)` with `T.dip[P].agree != 1`, after the
  all-goods affordability check, for each good both can use with
  `T.dip[P].offers[r] > 0`: `bucket −= o` (floor 0), `escrow −= o` (floor
  0), `tributes[r] += o`; for each good with a nonzero human attack price:
  the same with `price`, and `dows[r] = price`. At the exchange, per good
  both can use: `P.tributes[r] −= max(P.dip[T].offers[r], 0)`,
  `T.tributes[r] −= max(T.dip[P].offers[r], 0)`, then both floored at 0;
  the `dows` refund does not touch `tributes` (B). In `clear_agree(me, p)`
  with `agree == 1`: per good, `m = min(tributes, dows)` and then (offer
  `> 0`) `m = min(tributes, offers)` move back to the bucket.
- **Reset**: leader setup.
- **Readers**: `clear_agree`'s cap, the exchange clamps, the `LEADERS` dump
  (`log_data` `6e7515`).
- **Crate site**: none; needs the `Diplomacy` record, then an
  `action_respond` and `clear_agree` port beside `economy.rs`'s ledger
  (`escrow`, `economy.rs:687`, is debited by the commit).
- **Plain game**: needs a human or style 0/8/11; 0 throughout an
  all-computer style-1 game.
- **Confidence**: high, every store listing-read. Open: none on the rule;
  the run1086 zeros remain a reading until the packet check.

## Leaf 6: `tribute_sent` (+0x860, int)

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | writers `6d0f1c` (`P += g_P`), `6d0f8c` (`T += g_T`), unscaled, only for goods both can use | yes | yes | agree |
| 2 | reset `Leader::reset_score (006e37f0)` `6e3895`, from `Leader::init` (`:140`) and `Setup::build_game (005ac190)` | yes | yes | agree |
| 3 | escrow and `dows` refunds do not count | yes | implied | agree |
| 4 | reader `AchieveChartPane::set_chart_data (007f2c40)` only | yes | yes | agree |
| 5 | not in `rondata` coverage's leaf list | yes | — | one-sided; confirmed (`crates/rondata/src/diff/coverage.rs:295` lists `tributes[scan]`, not `tribute_sent`/`tribute_received`) |
| 6 | plain game as leaf 5 | yes | yes | agree |

### Settled

- **(1) spot-checked.** `6d0f19 movl -0x14(%ebp),%eax` (`P·0x6eec`),
  `6d0f1c addl %edi, 0xe3abf0(%eax)` (`edi = max(P.dip[T].offers[r],0)`,
  `6d0ee7`–`6d0ef2`); `6d0f85`–`6d0f8c` the same for T with `ebx`
  (`6d0ed0`–`6d0edb`). Both inside the two-`type_avail` gate (`6d0e9e`,
  `6d0eb1` → `6d0fe6`). **Confirmed.** No net (store, `lea`, absolute with
  base) finds another writer.

### Rule to port

`tribute_sent[giver] += max(giver.dip[receiver].offers[r], 0)` per good both
can use, at every executed exchange; 0 at `reset_score` (every leader
setup). Score-screen only. Crate: the exchange of an `action_respond` port;
no sim reader. Plain game: needs a human or style 0/8/11. Confidence high.

## Leaf 7: `tribute_received` (+0x864, int)

### Claims

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | writers `6d0f31` (`T += scale_tribute(P, g_P)`), `6d0faf` (`P += scale_tribute(T, g_T)`), the giver's scale | yes | yes | agree |
| 2 | `scale_tribute (006d5240)`: `pct = epoch[2]·commerce_tribute + base_tribute`, floor 1; `> 100` or `== 100` unscaled; else `x·pct/100` with `+0`/`+50`/`+99` below 20 / below 100 / from 100 | yes | yes | agree |
| 3 | reset `reset_score` `6e389b` | yes | yes | agree |
| 4 | reader `set_chart_data` only | yes | yes | agree |
| 5 | the crate loads neither constant | yes | — | one-sided |

### Settled

- **(1) spot-checked.** `6d0ef5`–`6d0ef6`: `scale_tribute` called with
  `ecx = &leaders[P]` (`-0x10`) on `g_P`; its result is the bucket credit
  (`6d0f0b`) and, recomputed at `6d0f2c` on the same `ecx`, the add at
  `6d0f31` into `esi = T·0x6eec`. `6d0f54`–`6d0f62` and `6d0f92`–`6d0faf`
  the same with `ecx = &leaders[T]` into `-0x14` (`P·0x6eec`). The giver's
  scale. **Confirmed.**
- **(5) confirmed.** `grep -rn commerce_tribute crates/` finds nothing.
- **(2)**: both readings agree from the decompile; the rounding branches
  were not re-listed here. The load scale of the two `Constants` fields is
  open in both.

### Rule to port

`tribute_received[receiver] += scale_tribute(giver, max(giver.dip[receiver]
.offers[r], 0))` per good both can use, at every executed exchange; the same
integer the receiver's bucket gains. 0 at `reset_score`. Score-screen only.
Crate: the exchange of an `action_respond` port, plus `scale_tribute` and the
two constants (not loaded today). Plain game: needs a human or style 0/8/11.
Confidence high on the stores; medium on `scale_tribute`'s constants' scale.

## Against the README row

The row names "the `action_*` family, `consider_tribute` and
`Leader::diplomacy`'s arms". Corrected: of the `action_*` family only
`Leader::action_respond (006d03c0)` stores a leaf (both readings);
`Leader::clear_agree (006d1af0)` (pointer store into `tributes`),
`Leader::process_taunt (006b8cc0)` (`gift_stamp`), `Leader::init
(006e3930)` (five leaves, unconditional loops) and `Leader::reset_score
(006e37f0)` (two) are missing from it. `Leader::diplomacy` stores only
`counteroffer` (4) and `tribute_demanded` (1). My own net (write-shaped
instructions with a destination in any leaf's range, absolute or
register-relative, over the whole listing, filtered to `Leader*` classes,
plus the four absolute-with-base adds at `0xe3abf0`/`0xe3abf4`) finds no
writer outside this list.

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| shared (A) all-computer style 1 | 1 / 0 / 0 | no event writer is reached (no deal is ever originated: `6be556`, `6c09c8`, `6c0a39`); only setup zeroing, so every leaf is 0 and the crate's stubs are exact |
| shared (B) `dows` refund | 1 / 0 / 0 | an executed deal refunds a human's attack price to the bucket and leaves it in `tributes` until setup |
| shared (C) `P.dip[P].dows` | 1 / 0 / 0 | the stray store writes 0 over 0; the tail `clear_all(P.dip[T])` (`6d12d5`) zeroes the real slot |
| `tribute_stamp` | 10 / 0 / 0 (+1 settled by A) | `R.stamp[G] = frame` at an exchange give `> 0` when R human, or give `>= 100·G.blacken`, R's stock `< 3000`, and WEALTH or `econ & 4`; 0 at setup |
| `gift_stamp` | 9 / 0 / 0 (+1 by A) | `G.gift[R] = frame` at every positive give; also on a taunt grant when unlocked or neutral; read by the taunt refusal and ally pricing (4500 frames) |
| `counteroffer` | 10 / 1 / 1 | 1 when a computer's shortfall − 99·[set] `> 0` on a deal asking it nothing; 0 on its accept/reject and on any `action_respond(me,k,*)` entry |
| `tribute_demanded` | 12 / 2 / 2 | 1 after a computer's demand (style 0/8/11; war instead if `k.blacken >= 4` or `k.WEALTH < (epoch+1)(k.blacken+1)·100`, ...); 0 on its own `action_respond(me,k,*)` |
| `tributes` | 8 / 1 / 0 (+2 by B, C) | `+o`/`+price` on agreeing, `−max(offer,0)` at exchange floored 0, `−min` refunds on withdrawal; dumped by `log_data` `6e7515` |
| `tribute_sent` | 6 / 0 / 0 | `+= max(offer,0)` per good both can use at the exchange; 0 at `reset_score` |
| `tribute_received` | 5 / 0 / 1 | `+= scale_tribute(giver, give)` at the exchange; 0 at `reset_score`; constants' scale open |

Counting the one-sided `Leader::init` reset claims as confirmations for the
first reading: corrected claims are `counteroffer` (4), `tribute_demanded`
(6, 9) and `tributes` (8). The opens are `counteroffer`'s `6c035d` arm,
`tribute_demanded`'s `-0x30` and epoch index, and `scale_tribute`'s
constants' load scale. None of them changes a store.
