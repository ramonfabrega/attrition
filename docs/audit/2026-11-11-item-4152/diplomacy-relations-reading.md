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

