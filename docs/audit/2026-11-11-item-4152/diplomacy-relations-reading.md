# Item 4152, family 7 — `diplomacy-relations`: first reading

- **Family**: 7, `diplomacy-relations`.
- **Leaves** (`LeaderData` offsets; `int[8]` unless named): `good_deeds`
  (+0xd4), `blacken` (+0x20c, `int`), `dow` (+0x270), `broke_alliance`
  (+0x2b0), `made_peace` (+0x2d0), `last_spoke` (+0x2f4), `hire_stamp`
  (+0x1b4), `hire_who` (+0x1d4).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), first reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), the listing
  of `riseofnations.exe` (`llvm-objdump`), `types.txt`. `Leader` is
  `0x6eec`, `LeaderData` `0x6ee4`.

## Ground the whole family stands on

- **`Leader` puts `LeaderData` at offset 0.** Every `Leader` method in the
  export prints these leaves at their raw `LeaderData` offsets
  (`Leader::init (006e3930)` line 153, `&this->field_0xd4`), and
  `Leaders::clear (006ed5d0)` clears from the slot base. So a store off a
  `Leader*` reads `0x<off>(%reg)`, and one off the global array reads
  `0xe3a390+off(...)`.
- **`diplos` (`+0x74`): 0 war, 1 peace, 2 alliance.** `LeaderData::is_enemy
  (006ebaa0)` is true when either side holds 0; `LeaderData::is_ally
  (006edb50)` needs 2 on both sides, or the leader's own slot.
- **`leader_flags & 4` is "human".** `LeaderData::is_human (006ec170)`
  returns exactly that bit. That matters for every leaf below:
  `action_declare`'s third argument is `~(flags >> 2) & 1` at four call
  sites, so it is **1 for a computer and 0 for a human**. (Side note for
  the adjudicator: `crates/sim/src/combat/take_damage_view.rs:32` names
  that bit `LEADER_OUT`, "the player is out". It is `is_human`. That
  file's arithmetic is unchanged by the name.)
- **`ALLY_TO_WAR_DELAY` and `ALLY_TO_WAR_GRACE` load as −1 from shipped
  data.** Both come from `Constants::get_item (0057fa60)`, which returns
  `get_attrib_num(…, -1)` when the tag is missing (`Constants::init
  (00569a90)` lines 2398–2401). The two names appear in the install only in
  `internal_strings*.xml`, never in a rules file (grep of
  `game/data/*.xml`). With a delay of −1, every
  `frame − stamp < delay` test is false. This agrees with
  `crates/sim/src/territory/attrition.rs:431–435`.
- **Method.** Absolute stores were found by scanning the full listing
  (`llvm-objdump -d` of the executable) for every displacement inside each
  leaf's `0xe3a390+off` range. Register-relative stores were found by
  scanning every function the export types with a `Leader*`, `LeaderData*`
  or `LeaderOut*` (937 functions) for displacements in the leaf's range.
  Each hit off a register was then checked against the function to see
  whether the base is a leader. The decompile was grepped for both
  `field_0x<off>` and the PDB name.

## `dow` (+0x270, int[8]) and `blacken` (+0x20c, int)

One writer steps both, so they share a section.

### Writers

- **`Leader::action_declare (006dab50)`** is the only runtime writer. Its
  stores are `incl 0x20c(%esi)` at `6dad38` and `6dad5b`, and
  `incl 0x270(%esi,%edi,4)` at `6dad3e` and `6dad61`, where `esi` is
  `this` (a `Leader*`) and `edi` is `param_1`, the target slot.
- **Zeroed by `Leader::init (006e3930)`.** `blacken` is cleared at line 170
  (`6e3be0`). `dow[u]` is `puVar23[0x67]` off `+0xd4`, inside the per-slot
  loop at lines 178–244 (`0xd4 + 0x67·4 = 0x270`).
- **Zeroed by `Leaders::clear (006ed5d0)`**, which memsets the first
  `0x6929` bytes of each slot (`6ed5e0`–`6ed5ee`).
- **Nothing else stores.** The listing scan found no other store in either
  range: absolute `0xe3a59c` and `0xe3a600..0xe3a61f`, or `0x20c`/`0x270`
  off a register in a leader-typed function. The other register-relative
  hits are other records' fields: `Leaders::clear`'s `0x278`–`0x288` off
  `esi`, which are the `Diplomacy` records' bitsets after the memset, and
  `ObjectType`, `Objects` and `River` fields.

### Event, step and predicate, read from the listing

`action_declare(target, state, free, force)` returns early, with nothing
written, when the declarer is already an enemy of `target` (`6dab63`). So
it never makes peace out of war. With `force == 0`, which is what every
call site passes except `DropControl::process_drop (00959500)`, the cases
are as follows.

1. **`state == 0` and the pair is allied.** The call does not go to war.
   It goes round again with `state = 1` (`6dab88` → `6dac06`), so declaring
   on an ally **breaks the alliance down to peace**. The second pass skips
   the war-allowed test (`6dabe4`).
2. **`state == 0` and the pair is at peace.** In order:
   - It refuses (returns 1) when
     `max(this.broke_alliance[t], list[t].broke_alliance[me])` is non-zero
     and `frame − that < ALLY_TO_WAR_DELAY` (`6dab8d`–`6dabc5`). This never
     fires on shipped data (delay −1).
   - When `free == 0`, it refuses when `LeaderData::afford_dow (006d5ce0)`
     fails.
   - It refuses when `Game::war_allowed (00594670)` is false.
3. **The counting block, `6dacec`–`6dad61`.** It runs **only when
   `free == 0`**: `cmpl $0,0x10(%ebp); jne 6dad68` skips it, together with
   `pay_dow`. Inside it:
   - `Leader::pay_dow (006d2b10)(t, state)` runs, and again with state 0
     when the pair is allied and `state == 0`, which is the forced case.
   - **Unprovoked declaration.** If `state == 0` and (the pair is allied
     *or* `this.invaders[t] < 5`, read at `0x290+4t` at `6dad24`): when
     `dow[t] != 0`, `blacken += 1`; then `dow[t] += 1`.
   - **Alliance break.** If `is_ally(t)` still holds, because `set_diplo`
     has not run yet: when `dow[t] != 0`, `blacken += 1`; then
     `dow[t] += 1`.
4. **Then, free or not:** `broke_alliance[t] = frame` if still allied (see
   that leaf). Then `Leader::set_diplo (006ec6a0)(t, state)`, the cascade
   over each team (`Leader::ally_diplo (006d0120)`), and the console
   message.

So, in plain integers:

- **`dow[t]`** counts, by +1 per event, each **paid** declaration this
  leader made on `t`: a war declared from peace while `t` had fewer than
  5 counted invaders inside my land, and every alliance broken with `t`.
  A forced war on an ally counts 2.
- **`blacken`** counts, by +1, each such event against a slot whose `dow`
  was already non-zero. That is the second and later offence against the
  same leader, summed over all leaders.
- Units are events. There is no decay.

**The predicate that matters is `free`.** `Leader::diplomacy (006bc950)`
passes `free = 1` literally at all six of its `action_declare` calls
(decompile lines 1061, 1070, 1501, 1507, 1511, 3010). The sites that pass
`~(flags >> 2) & 1` pass 1 for a computer:

- `Object::take_damage (00652020)` line 131. Its victim must be
  non-human, so `free` is always 1 there.
- `Nuke::do_damage (0092bc80)` lines 195 and 324.
- `Leader::action_respond (006d03c0)` (`6d1273`, `6d1297`).

`ScenarioFuncSet::declare_war (009fbe80)` and `DropControl::process_drop`
pass 1. Only `CommandPackage::process_declare (009472b0)` passes
`free = 0`: the human's Declare button, `action_declare(who, whom, treaty,
0, 0)`.

**So `dow` and `blacken` move only for human declarers**: the Declare
command, or a human partner in an accepted joint-attack deal. A computer
leader's `dow` and `blacken` stay 0 for the whole game. The crate's own
note agrees for one human event. `crates/rondata/src/blind.rs:333–343`
(run1078, `TEAM_STYLE 0`) records the Declare button on frame 910,
`process_declare 1 0 911`, and leader 0's `dow[1]` going 0 → 1 on block
912.

### Readers

Every reader is in `Leader::diplomacy` except two, and in each one the
read leader is the **target `t`**, as in `list[t].dow[me]` or
`list[t].blacken`, apart from the own-slot reads.

- **`6bcf0a`.** `weak /= max(1, list[me].dow[me])`, indexed by
  `who·0x6ef0`, which is the leader's **own slot**. Nothing writes the own
  slot: every caller passes a target other than the declarer
  (`take_damage` checks `local_14 != param_8`, the hire arm skips
  `i == me`, and diplomacy's `t` is never me). So the divisor is 1.
  `crates/sim/src/leaders/diplomacy.rs:304–307` already has this.
- **`6bd871` and `6bd87e`.** In the peace arm against a **human** target,
  aggression is cleared by difficulty and by the target's wars only while
  `list[t].dow[me] < 2` **and** `list[t].blacken < 4`. A human who has
  twice declared on this computer, or who has four repeat offences in
  all, keeps the computer's aggression armed.
  `crates/sim/src/leaders/diplomacy.rs:397–417` reads both as 0.
- **`6bdc3b`.** The "their cities that were mine" coin (one in four, or
  one in two for a boomer) that clears aggression runs only while
  `list[t].blacken < 2`. That is `diplomacy.rs:444–451`.
- **`6be667`, `6be7ec`.** Two weights in the offers arm (decompile lines
  1934 and 1958). The weight carries a ×2 when the third party's
  `blacken != 0`.
- **`6bfcf4`, `6c017e`, `6c0b1a`.** The offers and at-war arm (lines 1343,
  1551, 2178, 2864):
  - the tribute demanded scales by `(blacken + 1)`;
  - a peace offer is refused when `blacken > 3`;
  - the alliance logic tests `blacken > 1`;
  - the console message text depends on `blacken == 0`.
- **`6bfdcd`.** Proposal valuation (line 2702). When `list[t].dow[me] < 2`
  the balance gains +100 and/or +100 by difficulty (`get_diff < 2`,
  `< 3`). When `< 3` and the proposal carries something, it gains
  `epoch·100 + 100`. These are byte offsets on a `uint*` in the decompile,
  so its `+0x19` reads as +100 in the listing at `6bfdf4`–`6bfe09`.
- **`6c00cd`.** Only picks the console's message text, from
  `internal_random`.
- **`6c0966`.** Another **own-slot** read, `list[x].dow[x] < 2` at
  `x·0x6ef0`, in the at-war arm before a coin. It is always true.
- **`Leader::consider_tribute (006d1360)` at `6d1396`.** For a **computer**
  receiver, `tribute_stamp[giver]` is stamped only when the amount is at
  least `100 × list[giver].blacken`, among other conditions (family 8).
- **`LeaderData::log_data (006e5110)`** logs `blacken` (line 619). That is
  dump only.

### Reset

- `Leaders::clear` and `Leader::init` zero both. There is no per-frame
  fold and no `reset_score` store: the listing scan found no other store.
- Save and scenario load copy the bytes in bulk. I did not read them.

### Crate

- **There is no `action_declare` event.** The closest is
  `Sim::set_diplo` (`crates/sim/src/leaders/relations.rs:116`), reached
  only by the golden replayer's console verbs (`crates/rondata/src/golden.rs:2147`
  → `Setup::set_diplo`, `crates/sim/src/setup.rs:253`), and
  `Sim::declare_war` (`relations.rs`, near line 292).
- **`take_damage_view.rs:195`'s `declare` is a view method with no live
  implementer.** It runs only in the sweep
  (`crates/sim/tests/sweep/object_take_damage_arms.rs`).
- **The readers are SEAMs in `crates/sim/src/leaders/diplomacy.rs`** at
  lines 304–307, 397–399 and 444–445.
- **Where a builder would add the counters:** a new
  `Sim::action_declare(me, t, state, free, force)` beside `set_diplo` in
  `leaders/relations.rs`. The human's Declare command and `set_diplo`'s
  console verb would route through it with `free = 0`, and
  `Leader::diplomacy`'s declarations with `free = 1`. Only the `free = 0`
  path would increment.
- **`crates/rondata/src/diff/leader.rs:47` lists `blacken` as "fog
  bookkeeping".** By this reading it is the human declarer's reputation
  count. The label is wrong, although listing it as unmodelled is right.

### Reached by a plain game?

- **Yes, for a human,** in any skirmish where a human and a computer are at
  peace or allied while war is allowed. Two examples:
  - **Team style 0, 8 or 11** (Diplomacy), where leaders start at peace.
    run1078 is exactly this.
  - **Age-based rush rules** (`rush_rules` ≤ 8) under team style 1. Peace
    starts while `RUSH_RULES` is above the starting age, and
    `end_the_peace` fires only for timed rules (`diplomacy.rs:74–77`), so
    after the age is reached the peace lasts until someone declares.
- **Locked-team style 1 with no rush rules starts at war,** so no
  `action_declare` call gets past its `is_enemy` head.
- **Computer leaders never move either leaf.**

### Confidence and what is not read

- **High** for the writer set, the step and the `free` predicate. These
  were read in the listing, and one event is diff-backed by run1078's
  `dow` note.
- **Medium** for the `invaders < 5` reading. The test is `cmpl $5,
  0x290(%esi,%edi,4); jge`, read directly, but what `invaders` counts
  belongs to another family.
- **Not followed:**
  - `ScenarioRead::load_leader_chunk (009ac080)` and
    `Leaders::walk_data (006e38e0)`, the bulk load and save paths;
  - the remaining body of the offers arm around `6be667` and `6be7ec`,
    which I identified only by address and weight shape;
  - `Leader::pay_dow`'s internals, which are a cost and not this family.

## `broke_alliance` (+0x2b0, int[8])

### Writers

- **`Leader::action_declare (006dab50)`** at `6dad7f`:
  `movl frame, 0x2b0(%esi,%edi,4)`, with `esi` = `this` and `edi` = target.
  This is the only runtime store. The absolute range `0xe3a640..0xe3a65f`
  has no store anywhere; its four hits are reads at `5e161c`, `5e162c`,
  `5e164b` and `6dab9d`.
- **Zeroed by `Leader::init`** (`puVar23[0x77]`, `0xd4 + 0x77·4 = 0x2b0`,
  line 236) and by **`Leaders::clear`**'s memset.
- **Not leader stores:** the `0x2b4`–`0x2c8` hits in `Leaders::clear`,
  `plan_strategy` and `create_units`. Those are `Diplomacy`, `UnitType` and
  `BuildType` words: their bases come from pointer loads, not from a
  leader slot.

### Event, step, predicate

- `broke_alliance[t] = game.frame` (in frames) when `action_declare` finds
  the pair **still allied** just before its `set_diplo`, that is, when the
  call breaks an alliance.
- It is written **whether `free` is 0 or 1**. The store sits after the
  counting block's skip target `6dad68`, so computers stamp it too.
- **Callers that reach it:**
  - `Leader::diplomacy`'s `action_declare(t, 1, 1, 0)` at lines 1501 and
    1507, the computer breaking an alliance;
  - any `state == 0` declaration on an ally, which is turned into
    `state = 1` (see `dow`);
  - the human's Declare button on an ally;
  - `ScenarioFuncSet::declare_war`.
- Only the declarer's own slot `[t]` is written; `list[t].broke_alliance[me]`
  is not.

### Readers

All of them compare `frame − max(mine[t], theirs[me])` with
`ALLY_TO_WAR_DELAY`.

- **`action_declare` (`6dab8d`–`6dabc5`)** refuses a war while the window
  is open.
- **`Unit::process_attrition (005e11a0)`**, decompile lines 164–168: the
  "recently broke the alliance" grace in which the border does not bite.
- **`Options::do_declare_war (007216b0)`** lines 223–232: the console
  refusal text.
- **`IFaceMainBase::draw_timers (0080b060)`** lines 761–775: the on-screen
  countdown.
- **On shipped data the delay is −1, so no reader can change an outcome.**
  The leaf is write-only in effect, as
  `crates/sim/src/territory/attrition.rs:431–435` already states for the
  attrition reader.

### Reset

`Leader::init` and `Leaders::clear` only. No per-frame fold.

### Crate

- No event. The alliance break would be the same new `action_declare`
  beside `Sim::set_diplo` (`crates/sim/src/leaders/relations.rs:116`).
- The crate's `set_diplo` level 1 (`relations.rs:131–134`) is the
  console's peace verb, not a declaration, so the stamp belongs on the
  declare path only.
- The allied arm of `Leader::diplomacy` that calls the break is a SEAM at
  `crates/sim/src/leaders/diplomacy.rs:389–392`.

### Reached by a plain game?

- **Yes**, whenever an alliance is broken in a skirmish. For a computer
  that is the allied arm of `diplomacy`; for a human, the Declare button
  on an ally.
- Locked teams (style 1) start allies on the same team, and whether the
  allied arm's break can fire under a locked style was not read; see
  below. An unlocked style (0, 8 or 11), with alliances made in game,
  reaches it.

### Confidence and what is not read

- **High** for the store and its predicate.
- **Not read:** whether the allied arm (decompile lines 1495–1511) is
  reachable under `TEAM_STYLE 1` between same-team computers. Its guards
  were not followed.

## `made_peace` (+0x2d0, int[8])

### Writers

- **`Leader::action_respond (006d03c0)`** at `6d108d` and `6d10a0`:
  `list[me].made_peace[p] = frame` and `list[p].made_peace[me] = frame`,
  where `me` = `this->who` and `p` = `param_1`. These are the only runtime
  stores. The absolute range `0xe3a660..0xe3a67f` has these two stores
  plus reads at `5e1697` and `5e16d0`.
- **Zeroed by `Leader::init`** (`puVar23[0x7f]`, `0xd4 + 0x7f·4 = 0x2d0`)
  and by **`Leaders::clear`**.
- The `0x2d8`–`0x2ec` hits off registers in `create_buildings` and the
  like are `BuildType` and `UnitType` fields.

### Event, step, predicate

- **The event is the deal-execution tail** of `action_respond` (from
  `LAB_006d0df9`, decompile lines 369–470).
- The deal's treaty term is `list[me].dip[p].treaty`, at
  `0xe40cc4 + me·0x6eec + p·0x5c`, i.e. `+0x6934`. Both sides are moved
  to `max(term, current)` by two `set_diplo` calls.
- **Then, if the term equals 1 exactly** (`cmpl $1,-0x24(%ebp)` at
  `6d1072`), both sides' `made_peace` are stamped with the frame.
- **The stamp tests the proposed term, not the outcome.** Because of the
  `max`, a peace term accepted between two allies leaves them allied and
  still stamps `made_peace`. A term of −1 (no treaty) skips the whole
  block (`6d1016`).
- I read the entry gating into the execution tail only in outline. It
  needs `param_2 == 1` (agree) and depends on the `Diplomacy` records'
  `agree` words (lines 74–80, 247–250 and 362–368). Within that reading,
  a pure gift executes at once, and a deal with a treaty or attack term
  executes once the other side's agreement is present.

### Readers

- **`Unit::process_attrition`** lines 171–174 is the only one: the same
  `ALLY_TO_WAR_DELAY` window, after peace.
- Inert on shipped data (delay −1).

### Reset

`Leader::init` and `Leaders::clear`.

### Crate

- No event. The crate has no `Diplomacy` record, no proposals and no
  `action_respond`.
- `Leader::diplomacy`'s offers arm, which calls `action_respond`, stops at
  the SEAMs at `crates/sim/src/leaders/diplomacy.rs:480–495`
  (`dip[t].agree` never 1; the offers arm and the at-war arm unported).
- A builder would add the stamp to a new `Sim::action_respond` in
  `crates/sim/src/leaders/` once proposals exist.

### Reached by a plain game?

- **Yes in principle**: a peace treaty accepted in a skirmish, between a
  human and a computer through the diplomacy window, or between computers
  through `diplomacy`'s offers.
- It needs a war between leaders that can make peace, which means an
  unlocked team style. Whether locked style 1 lets a peace term through
  was not read.
- It also runs under the scenario layer
  (`ScenarioFuncSet::offer_peace (009fc1f0)` and `accept_offer
  (009fc410)`) and from `CommandPackage::process_accept (00946fb0)`.

### Confidence and what is not read

- **High** for the two stores and the `== 1` predicate (listing).
- **Medium** for the entry gating into the execution tail.
- **Not followed:** `Diplomacy::clear_all`'s callers, and
  `Leader::process_taunt (006b8cc0)`'s call to `action_respond`.

## `hire_stamp` (+0x1b4, int[8]) and `hire_who` (+0x1d4, int[8])

One writer loop steps both, so they share a section.

### Writers

**`Leader::action_respond (006d03c0)`**, the hire loop at `6d11bc`–`6d12a9`.
For each slot `i`:

- `6d121e`: `list[me].hire_stamp[i] = frame` (absolute
  `0xe3a544(,me·0x1bbb+i,4)`).
- `6d122e`: `list[p].hire_stamp[i] = frame`. This is `-0x20(%ecx)`, where
  `ecx` walks `&list[p].hire_who[i]`, set up at `6d11d4` as
  `lea 0xe3a564(p·0x6eec)`.
- `6d123a`: `list[me].hire_who[i] = p`.
- `6d1244`: `list[p].hire_who[i] = me`.

Other stores:

- **`Leader::init`**, inside the per-slot loop: `hire_stamp[u] = 0` at
  `puVar23[0x38]`, and **`hire_who[u] = −1`** at `puVar23[0x40]`
  (`0xd4 + 0x40·4 = 0x1d4`, decompile line 189).
- **`Leaders::clear`** memsets both to 0 *before* init sets `hire_who`
  to −1.
- **Nothing else stores.** The absolute ranges `0xe3a544..0xe3a563` and
  `0xe3a564..0xe3a583` have just these stores plus reads at `6bdd57`,
  `6bdd6b` and `6d11d4` (a `lea`).
- **`BuildOut::make_options (00629ed0)`'s `pLVar16->hire_who[4]`** (lines
  241, 457 and 592) is the export mistyping a `UnitType*` as
  `LeaderData*`. It reads `+0x1e4` of a unit type (the `0x4000000` flag),
  is not a leader access, and is a read anyway.
- **The `0x1bc`/`0x1dc` stores in `Leader::init`** at `6e3d57`/`6e3d5d`
  are the same loop's `hire_stamp[2]`/`hire_who[2]` slots after unrolling.
  They are already covered.

### Event, step, predicate

Two leaders `me` (the responder, `this`) and `p` (`param_1`) execute an
accepted deal in `action_respond`'s execution tail (see `made_peace` for
the gating). Then, **only if `Game::war_allowed`** (`6d11c2`), for every
slot `i` in 0..8 with `i ≠ me`, `i ≠ p`, and the deal's attack term for
`i` non-zero (`list[me].dip[p] + 0x3c + 4i`, absolute
`0xe40cf8 = 0xe3a390 + 0x6968`; set by `Leader::action_attack (006d14e0)`
in both parties' records):

- both parties stamp `hire_stamp[i]` with the frame;
- each records the other as `hire_who[i]`;
- each party that is not yet at war with `i` calls
  `action_declare(i, 0, free = !human, 0)`.

So `X.hire_who[i] = Y` means "X and Y agreed to attack `i` together", and
`X.hire_stamp[i]` is the frame of that agreement. There is no liveness
test on `i`. Units are frames and a slot index.

`Leader::action_propose_attack (006d14d0)` is a single
`jmp 0x6d14e0`, so it is `action_attack`. `action_attack` has three
callers:

- `Leader::diplomacy` at `6c0f2b`, decompile lines 2309–2314: the
  computer proposes alliance plus attack, then calls
  `action_respond(t, 1)` on its own offer;
- `CommandPackage` at `946c32`, the human;
- `ScenarioFuncSet` at `9fc373`.

The computer's proposal arm is skipped for any team style other than 0,
8 or 11 (decompile lines 2178–2193, `goto LAB_006c0f50`).

### Readers

All are in `Leader::diplomacy`, for computer `me` against target `t`.

- **`6bdc8x`–`6bdca9`**, decompile lines 925–928: if
  `me.hire_stamp[t] ≠ 0` and `frame − me.hire_stamp[t] < 13500`
  (`0x34bc`), then `aggression[t] = 1`. I agreed to attack `t`, so I stay
  armed against `t`.
- **`6bdcdf`–`6bdd04`**, lines 929–937: the "my cities that were theirs"
  coin (one in three, one in two for a boomer) clears aggression only if,
  besides the coin, `me.hire_stamp[t] == 0` or
  `frame − me.hire_stamp[t] > 4500` (`0x1194`). The coin is drawn whether
  or not the stamp blocks the clear (`6bdcd3` comes before `6bdce2`).
- **`6bdd30`–`6bde0f`**, lines 940–970, a loop over live `l ∉ {me, t}`:
  - **(a) Dead as compiled.** "If `list[l].hire_who[me] == l` and
    `frame − list[l].hire_stamp[me] < 9000`, set `aggression[t] = 1` and
    stop". The compare is `cmpl %edi, 0xe3a564(,l·0x1bbb+me,4)` at
    `6bdd57`, and `edi` is the loop counter `l` (`xorl %edi,%edi` at
    `6bdd12`, `incl %edi` at `6bddf8`). The writer only ever stores the
    *partner*, so `X.hire_who[i] ≠ X` always, and init is −1. The arm
    can never fire. (The intended operand was presumably `t`, "`l` and
    `t` agreed to attack me", but the binary says `l`.)
  - **(b) Live.** If my ally `t` (`is_ally(me, t)`, `6bdddb`) agreed with
    me to attack `l` (`me.hire_who[l] == t`, `6bddbe`) at most 9000
    frames ago (`frame − me.hire_stamp[l] ≤ 9000`, `6bddcd`, `jg` skips);
    and I am at war with `l` (either side's `diplos` 0, `6bdda6`–`6bddb9`);
    and `t` is **not** at war with `l` (both `list[t].diplos[l]` and
    `list[l].diplos[t]` non-zero, `6bdd87`–`6bdda4`; the `l == t` escape
    at `6bdd83` is dead because the loop skips `l == t`); then
    `aggression[t] = 1`.
  - In words, (b) says: an ally who promised to join my war and has not
    done so becomes a target.
- **Offers arm, line 1345**: `hire_stamp[t]` less than 13501
  (`0x34bd`) frames old is one of the conditions that sends the
  tribute-demand branch elsewhere. Not decoded further.

### Reset

`Leader::init` (`hire_stamp` 0, `hire_who` −1) and `Leaders::clear` (both
0). No fold.

### Crate

- No event.
- **Reader SEAM** at `crates/sim/src/leaders/diplomacy.rs:453–455`
  ("hiring is not modelled, so neither of their two arms fires").
- **There are three live reader arms, not two:**
  1. the 13500-frame rearm;
  2. the 4500-frame guard on the mine coin;
  3. the loop's arm (b).

  Arm (a) is dead.
- A builder would add the writer to a new `Sim::action_respond`, the deal
  execution, in `crates/sim/src/leaders/`. It needs `Diplomacy` records,
  which the crate does not carry.

### Reached by a plain game?

- **Only with Diplomacy on** (team style 0, 8 or 11). There, a computer's
  alliance-plus-attack offer, or the human's joint-attack proposal through
  the diplomacy window, can execute. It needs three or more players,
  because `i ∉ {me, p}`.
- **Not under locked team style 1**, as far as the computer's offer goes.
  Whether the human's window offers attack terms under style 1 was not
  read.
- Scenario: `ScenarioFuncSet`.

### Confidence and what is not read

- **High** for the four stores, the init value −1 and the dead arm (a).
  All were read in the listing.
- **Medium** for the gating into the execution tail.
- **Not followed:** `CommandPackage` at `946c32`'s argument mapping;
  `Diplomacy::num_attacks`.

