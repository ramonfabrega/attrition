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

