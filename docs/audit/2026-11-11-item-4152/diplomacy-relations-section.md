## 7. Diplomacy: relations, declarations and hires

### 7.1 Leaves

- `good_deeds`, +0xd4, `int[8]`, by target; the valuation's own units.
- `hire_stamp`, +0x1b4, `int[8]`, by the hired-against slot; a frame.
- `hire_who`, +0x1d4, `int[8]`, by the hired-against slot; a slot, −1 none.
- `blacken`, +0x20c, `int`; a count of repeat declarations.
- `dow`, +0x270, `int[8]`, by target; a count.
- `broke_alliance`, +0x2b0, `int[8]`, by target; a frame.
- `made_peace`, +0x2d0, `int[8]`, by the other party; a frame.
- `last_spoke`, +0x2f4, `int[8]`; always 0.

`frame` is the sim frame. `Leader::init (006e3930)` resets every leaf for
every slot (`Leaders::init` calls it for all ten, with no exit before its
per-slot loop): all to 0, except `hire_who[j] = −1`. `Leaders::clear`'s
memset leaves 0 in all, `hire_who` included, until the next `Leader::init`.
`Leader::reset_score (006e37f0)` touches none. Nothing decays.

### 7.2 The rule

Three writers own the seven live leaves: `Leader::action_declare
(006dab50)` for `dow`, `blacken` and `broke_alliance`;
`Leader::action_respond (006d03c0)` for `made_peace`, `hire_stamp` and
`hire_who`; `Leader::diplomacy (006bc950)` for `good_deeds`.
`LeaderData::log_data (006e5110)` prints all eight, so all are in the
`LEADERDATA` dump.

**`action_declare(me, t, level, quiet, force)`**, in order:

1. return if `t` is already an enemy;
2. turn an allied pair's request into level 1;
3. refuse inside `ALLY_TO_WAR_DELAY` (signed); call `afford_dow` only when
   `quiet == 0`; refuse, or force level 1, on `war_allowed`;
4. **the counting block, only when `quiet == 0`:**
   - block A: if `level == 0` and (`allied(me, t)` or `invaders[me][t] <
     5`, signed): `if dow[me][t] != 0 { blacken[me] += 1 }`, then
     `dow[me][t] += 1`;
   - block B: if `allied(me, t)` still holds: the same pair again.

   `blacken` reads `dow` before that block's own increment. A forced level 0
   on an ally would count twice; no caller reaches it, since the only
   `force = 1` caller, `DropControl::process_drop`, passes `quiet = 1`;
5. **for any `quiet`:** if the pair is still allied both ways,
   `broke_alliance[me][t] = frame`. The target's mirror slot is not written;
6. `set_diplo(me, t, level)`.

**Who passes `quiet = 0`**, from all 16 call sites: only the human's
Declare (`CommandPackage::process_declare`, 9473aa) and a human party's leg
of the hire loop below. `Object::take_damage` and `Nuke::do_damage` pass
`!(declarer & 4)`, but each skips a human declarer before the call, so they
never pass 0. `Leader::diplomacy`, `action_respond`'s treaty call
(6d04ed), `DropControl` and `ScenarioFuncSet::declare_war` pass 1. So
`dow` and `blacken` move only for a human declarer. `broke_alliance` moves
on any break through `action_declare`: `diplomacy` 6bed1c, a level-0 call
on a mutual ally, 6d04ed, the human's Declare, `DropControl`. The 6beceb
call is dead (it needs `t == me`, which the per-target loop skips). The
original's `Game::do_frame` peace-ending and the console verb call
`set_diplo` directly and must not stamp.

**`action_respond(responder, proposer, response)`**, apply arm, after the
goods exchange (§8):

- **Treaty.** `w = responder.dip[proposer].treaty` (+0x6934). If `w != −1`:
  `set_diplo(responder, proposer, max(w, diplos[responder][proposer]))`,
  then the same for `(proposer, responder)`, both signed. If `w == 1`, both
  `responder.made_peace[proposer]` and `proposer.made_peace[responder] =
  frame`, even when the `max` keeps the pair allied.
- **Hires**, reached with or without a treaty, only if `war_allowed`. For
  each `X` in 0..8, `X ∉ {responder, proposer}`, with
  `responder.dip[proposer].attacks[X] != 0` (+0x6968):

      hire_stamp[responder][X] = hire_stamp[proposer][X] = frame
      hire_who[responder][X] = proposer
      hire_who[proposer][X]  = responder

  then responder, then proposer, if not already an enemy of `X`,
  `action_declare(party, X, 0, quiet = !human(party), 0)`. The stamps do
  not depend on the declarations. `attacks[]` is written into both records
  by `Leader::action_attack (006d14e0)`.

**`good_deeds`**, in `diplomacy`'s offers arm (`me` computer, `t` other):

- The valuation's score `S` is seeded with `good_deeds[me][t]` and
  accumulates the deal (resource value × amount >> 8, flat 25/50/100 steps).
  An early-game multiplier on asked goods applies: ×4 before frame 4500,
  ×3/2 (truncating) before 9000 when the pointer at `-0x40` reads 0. The
  over-ask flag `-0x50` is set when an ask exceeds my decrypted stock.
- `need = −S`; under `treaty == 2`, `need = (my epoch + t.blacken + 1) ×
  (100, or 50 under semaphore[0] & 4) × ((*-0x48 != 0) + 1) − S'`, with
  `S' = S + epoch[t]·{0, 50, 100}` plus adjustments. Then `need −= 99` if
  `counteroffer[me][t] != 0`.
- Accept, on `need <= 0` with the over-ask flag clear: `counteroffer[t] =
  0`; **only if `any_proposals(t)`**: `action_respond(t, 1)`, `need += 50`
  if `weak[t] != 0` and `!is_tribute_period(leaders[t], me)`, then
  `good_deeds[me][t] −= need`. Without a pending proposal it returns
  before the store.
- So the balance enters twice: an accepted deal leaves `2·good_deeds +
  surplus (+99 for a pending counteroffer) (−50 when weak)`. Port it
  doubled; it is what the listing does.

**`last_spoke`** has no writer and no sim reader; carry 0.

**Readers that change a decision.**

- `dow`: own diagonal `dow[me][me]` floored at 1 divides `weak` (6bcf0a);
  `leaders[t].dow[me] < 2` gates the peace arm for a human target (6bd871);
  `< 2`/`< 3` lift the alliance valuation by +100s (6bfdcd); `!= 0` picks a
  taunt (6c00cd). 6c0966 reads the diagonal too (always true for a
  non-self target).
- `blacken` (always `leaders[t].blacken`): gates `< 4` (peace arm 6bd87e,
  demand arm 6be667), `< 2` (race coin 6bdc3b), `>= 2` (alliance logic
  6c0b1a), `== 0` (text only, 6c017e); factors `(blacken + 1)` in the demand
  size `(25·epoch + 25)·(blacken + 1)` (6be7ec) and the alliance `need`
  (6bfcf4); `(blacken != 0) + 1` in two cost terms (6bf8d5, 6bf940);
  `consider_tribute`'s `100·blacken` floor for a computer receiver (§8).
- `hire_stamp`/`hire_who`: `me.hire_stamp[t] != 0 && frame − it < 13500`
  forces `aggression[t] = 1` (6bdc8a); the mine coin, drawn first, clears
  aggression only if `hire_stamp[t] == 0` or `frame − it > 4500` (6bdce2);
  `t` my ally, `me.hire_who[l] == t`, `frame − me.hire_stamp[l] <= 9000`, I
  at war with `l` and `t` not, forces `aggression[t] = 1` (6bdd87);
  `hire_stamp[t] != 0 && frame − it <= 13500` diverts the demand arm to war
  (6be680, §8). The arm at 6bdd57 compares `leaders[l].hire_who[me]` with
  `l` itself, a test no deal can pass; port it as written.
- `broke_alliance`, `made_peace`: `frame − max(mine[t], theirs[me])` against
  `ALLY_TO_WAR_DELAY` in `action_declare`'s refusal and `Unit::process_attrition
  (005e11a0)`; also `Options::do_declare_war (007216b0)` and
  `IFaceMainBase::draw_timers (0080b060)`. `ALLY_TO_WAR_DELAY` and
  `_GRACE` load −1 (`Constants::init (00569a90)` defaults; no shipped XML
  names them), so no outcome depends on either leaf; both are dump and
  checksum words a port still writes.
- `good_deeds`: only the next valuation's seed.

### 7.3 In the crate

- **No writer exists.** No `action_declare`, `action_respond`,
  `Diplomacy` record or hire model. `Sim::set_diplo` is
  `crates/sim/src/leaders/relations.rs:116`, `Sim::declare_war` `:294`.
  The counting and `broke_alliance` go in a new `Sim::action_declare` beside
  `set_diplo`, never in `set_diplo` (whose `_` arm an accepted treaty also
  calls), and never in `end_the_peace` or the console verb.
- **SEAMs** in `crates/sim/src/leaders/diplomacy.rs`: `dow` at `:305`,
  `dow`/`blacken` at `:398`, `let blacken = 0` at `:444–445` (the race
  coin), the hire readers at `:453`, the offers arm (`good_deeds`) at
  `:490` inside `diplomacy_with` (`:174`).
- **Traps.** `diplomacy.rs:453–454` says two hire arms; three read the
  words there (a fourth, 6be680, is in the demand arm).
  `crates/rondata/src/diff/leader.rs:47` labels `blacken` "fog
  bookkeeping"; it is a human's repeat-declaration count.
  `crates/sim/src/combat/take_damage_view.rs:34` names `0x4` `LEADER_OUT`;
  it is HUMAN (its uses are right: "skip a human declarer").
- **Checksum.** Rows `crates/sim/src/checksum/leaders.rs:60, 67, 68, 75,
  79, 81, 82, 84`, answered by `leader_long`'s `_ => None` (`:1305`). Each
  is an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1767, 1768, 1784, 1792,
  1795, 1796, 1806, 1812`; lines and counts read from the code, not the
  verdicts). The answer is a constant for all eight in an all-computer
  style-1 game: 0, and `hire_who` −1 per slot after `Leader::init`.

### 7.4 Reached by a plain game

**An all-computer skirmish at team style 1 reaches none of the event
writers.** No `action_respond` is ever called there (§8.4), so
`made_peace`, `hire_stamp`, `hire_who` and `good_deeds` (which needs
`any_proposals`) stay at their reset. No computer passes `quiet = 0`, so
`dow` and `blacken` stay 0. `broke_alliance` is open (7.6).

With a human: his Declare counts `dow`, and a second counted declaration on
the same slot counts `blacken` (run1078, style 0, shows `dow[1]` 0 → 1). A
human proposal or answer reaches `made_peace` and `good_deeds`; an executed
attack deal with three or more leaders and war allowed reaches the hires,
where a human party's declaration also counts. Team styles 0/8/11 reach the
deal writers without a human. `ScenarioFuncSet::*` is triggers. No Conquer
the World or multiplayer path is needed.

### 7.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/diplomacy-relations-*.md`); no diff backs
it yet. Verdicts: 87 confirmed, 12 corrected, 2 open.

Corrections that changed a rule:

- `take_damage` and `Nuke::do_damage` never pass `quiet = 0`: only the
  Declare command and a human hire party count `dow`/`blacken`.
- 6c0966 reads the diagonal `dow[t][t]`, not `dow[t][me]`.
- `blacken`'s readers re-mapped by address: 6be667 is a `< 4` gate and
  6be7ec the demand size; the `(blacken != 0) + 1` terms are 6bf8d5/6bf940;
  6bfcf4 is the alliance `need`.
- `diplomacy` 6beceb is dead; the live explicit break is 6bed1c.
- The 6bdd57 hire arm is a test that cannot pass, not a live third-leader
  rule; 6bf7a5 reads `diplos[k]`, not `hire_who`.
- `good_deeds` is in the dump.

Confidence is **high** on every store, gate and reset, each read in the
listing. It is **medium** on the full composition of `good_deeds`'s
valuation `S`, not walked term by term.

Witness, **not a pinned diff**: run1078 (style 0, a human Declare), where
the dump's `dow[1]` moves 0 → 1 (`crates/rondata/src/blind.rs:333–343`). No
`rondata::diff` row compares `dow`.

### 7.6 Not established

- **Declare on a locked teammate.** Whether the interface offers it under
  style 1. If it does, a human's Declare on an ally passes, becomes level 1,
  counts `dow` once (block B) and stamps `broke_alliance`. Reading
  `IFaceDiplo*`'s Declare enabling, or one capture, settles it. A self-target
  would also count the diagonal; no sim path sends one.
- **`broke_alliance` at all-computer style 1.** 6bed1c is in the demand arm,
  gated off at style 1. Whether the level-0 sites 6be1ef, 6be210 (a target
  not at war, rush rules) or 6c04df can name a mutual ally was not traced.
  Reading 6be067–6be1ad's entry, or a capture with a team and rush rules,
  settles it.
- **`good_deeds`'s `S`** term by term, and the scenario fixed-answer mode
  that skips it (`semaphore[2] & 2` read; the `semaphore[1] & 0x10` half
  not); both belong to the offers-arm port.
- **Which offers carry `treaty == 1`** (`action_treaty`, `action_offer`):
  §8's port.
- `walk_data` and `load_leaders_chunk` copy `last_spoke` and the rest on a
  load; neither originates a value.
