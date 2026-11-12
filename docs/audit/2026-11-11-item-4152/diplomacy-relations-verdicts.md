# Item 4152, family 7 (`diplomacy-relations`): adjudication

- Slug: `diplomacy-relations`. Leaves (`LeaderData` offsets, `int[8]` unless
  named): `good_deeds` (+0xd4), `hire_stamp` (+0x1b4), `hire_who` (+0x1d4),
  `blacken` (+0x20c, int), `dow` (+0x270), `broke_alliance` (+0x2b0),
  `made_peace` (+0x2d0), `last_spoke` (+0x2f4).
- Readings: `diplomacy-relations-reading.md` (first, "R1"),
  `diplomacy-relations-second.md` (blind second, "R2").
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text` listing
  (`exe.s` in the job scratch, grepped by address and displacement),
  `crates/sim` grep. Nothing run of the original. Leaves are appended as
  settled; the verdict table closes the file.
- Context taken from the sibling adjudication `diplomacy-tribute-verdicts.md`
  §(A): at team style 1 with no human, `Leader::diplomacy (006bc950)`
  originates no deal, no `Diplomacy` record leaves `clear_all`, and no
  `Leader::action_respond (006d03c0)` call is ever made. Re-used below where a leaf's
  writer sits behind an accepted deal; not re-derived.

## Shared ground, checked once

- **Base and stride.** Agreed by both and confirmed: a `Leader*` points at
  the `LeaderData` base, so `0xNNN(%reg)` off `this` is the leaf offset, and
  the global form is `0xe3a390 + slot·0x6eec + off`, the pair form
  `(slot·0x1bbb + other)·4`. A `·0x6ef0` stride (`0x6eec + 4`) is the
  **diagonal** `leaders[x].leaf[x]`; that settles one disagreement below.
- **My nets.** (1) Every operand of the whole `.text` listing whose constant
  lies in `[0xe3a390, 0xe3a390 + 10·0x6eec)` and reduces mod `0x6eec` into a
  leaf; (2) every write-shaped instruction (`mov/add/sub/inc/dec/and/or/xor/
  neg/not/shift/set/xchg/stos/pop`) whose destination displacement falls in
  a leaf's range inside any function the export names `Leader::`,
  `LeaderData::`, `Leaders::` or `LeaderOut::`; (3) for `last_spoke`, every
  access (read or write) at `0x2f4..0x313` in those classes and every
  `$0x2f4`/`$0xe3a684` immediate. Net (1) finds exactly the absolute stores
  both readers list (`6d108d`, `6d10a0`, `6d121e`, `6d123a`) and the `lea`
  `6d11d4` that carries the two pointer stores (`6d122e`, `6d1244`); net (2)
  finds exactly `6dad38`, `6dad3e`, `6dad5b`, `6dad61`, `6dad7f`, `6c08ae`
  as runtime stores plus `Leader::init`'s loop, the rest being other bases
  (the encrypted block's `0xdc..0xe8` in `gain_tech`/`lose_tech`/`init`,
  `Leaders::clear`'s BitMask words `0x278..0x288`/`0x2b8..0x2c8`,
  `plan_strategy`'s `movl 0xd4(%ecx)` off a pointer table and `movw
  0x2fc(%ecx)`, `LeaderOut::warn_resources`' `0x2cc`/`0x2d0` off the
  console object `[0xc06210]`). **No writer either reader missed.**
- **`Leader::init (006e3930)`'s per-slot loop** (`6e3c0d`–`6e3d93`) runs
  with `esi = &this->good_deeds[j]` (`leal 0xd4(%ebx)` at `6e3b74`, `+4` at
  `6e3d8d`). Its stores therefore land at `0xd4 + disp`: `(%esi)` 6e3c10
  `good_deeds[j]=0`; `0xe0` 6e3c2d `hire_stamp[j]=0`; `0x100` 6e3c36
  `hire_who[j]=-1`; `0x19c` 6e3d44 `dow[j]=0`; `0x1bc` 6e3d57
  `invaders[j]=0` (`+0x290`, not this family); `0x1dc` 6e3d5d
  `broke_alliance[j]=0`; `0x1fc` 6e3d63 `made_peace[j]=0`; `0x220` 6e3d6f
  `last_spoke[j]=0`. `blacken` is zeroed outside the loop, `6e3be0
  movl %ecx, 0x20c(%ebx)`. `Leaders::init`'s walk (`6ed8be`–`6ed8dd`) calls
  `Leader::init` for all ten slots, and `Leader::init` has no exit before the
  loop (only forward branches in `6e3930`–`6e3b74`), so **every slot's
  `hire_who` is −1 after setup**. `Leader::reset_score (006e37f0)` touches no
  leaf of this family (its stores are `0x18..0x44`, `0x808..0x938`).
- **`LeaderData::log_data (006e5110)` prints every per-slot leaf.** Its
  per-slot block walks `ebx = &this->diplos[j]` (`leal 0x74(%edi), %ebx` at
  `6e55be`, `+4` at `6e6464`) and pushes `0x60(%ebx)` (good_deeds, 6e588e),
  `0x140(%ebx)` (hire_stamp), `0x160(%ebx)` (hire_who), `0x1fc(%ebx)` (dow),
  `0x23c(%ebx)` (broke_alliance), `0x25c(%ebx)` (made_peace), `0x280(%ebx)`
  (last_spoke, 6e6269), and prints `blacken` by name. So every leaf here is
  in the `LEADERDATA` dump (`crates/rondata/src/diff/coverage.rs:295` lists
  all of them), which is what a diff would compare.
- **`Leader::action_declare (006dab50)`'s body** (`this` declarer; `target, level,
  quiet, force` at `0x8..0x14(%ebp)`) was re-read instruction by instruction
  and both readers' step lists stand: `is_enemy` → return (6dab63); the
  ally→level-1 loop (6dab73–6dab8b, 6dac06); the `ally_to_war_delay` refusal
  (6dab8d–6dabc5, signed `jl`); `afford_dow` only when `quiet == 0`
  (6dabc7–6dabdc); `war_allowed` refusal or forced level 1 (6dabe2–6dac00);
  the counting block only when `quiet == 0` (`6dacec cmpl $0,0x10(%ebp); jne
  6dad68`); the `broke_alliance` stamp at 6dad68–6dad7f for any `quiet`;
  then `set_diplo` (6dad99).
- **Every `action_declare` call site, arguments read from the pushes** (16
  sites; R2's table is confirmed in every row except the `quiet` reachability
  noted here):

  | caller (site) | level | quiet | force | can `quiet` be 0? |
  |---|---|---|---|---|
  | `Object::take_damage` 6522ed | 0 | `!(declarer & 4)` | 0 | **no**: `652277 testb $0x4, flags; jne 6522f2` skips a human declarer (the declarer is the victim's owner) |
  | `Leader::diplomacy` 6be1ef, 6be210, 6bed2f, 6c04df | 0 | 1 | 0 | no |
  | `Leader::diplomacy` 6beceb | 1 | 1 | 0 | no; and **dead**: behind `cmpl 0x8(%edi),%esi; jne` (6becdb) with `esi` = the target, while the per-target loop skips `t == me` (`6bca78 je 6be98a`) |
  | `Leader::diplomacy` 6bed1c | 1 | 1 | 0 | no |
  | `Leader::action_respond` 6d04ed | 1 | 1 | 0 | no |
  | `Leader::action_respond` 6d1273, 6d1297 (hire loop) | 0 | `!(party & 4)` | 0 | **yes**, a human party |
  | `Nuke::do_damage` 92c502, 92c71e | 0 | `!(declarer & 4)` | 0 | **no**: `92c48b`/`92c6ae testb $0x4; jne` skip a human declarer |
  | `CommandPackage::process_declare` 9473aa | packet | **0** (`947392 pushl $0`) | 0 (`94737f`) | always (the interface's Declare) |
  | `DropControl::process_drop` 959761, 959850 | 0 | 1 | 1 | no |
  | `ScenarioFuncSet::declare_war` 9fbedb | 0 | 1 | 0 | no |

  So **`quiet == 0` reaches `action_declare` from exactly two places**: the
  human's Declare command, and a human party's leg of an executed
  joint-attack deal. R1 states this; R2's "a human unit damaging a
  non-enemy" and "a human nuke" are **corrected** (both callers refuse a
  human declarer before the call).
- **`ALLY_TO_WAR_DELAY`/`ALLY_TO_WAR_GRACE` load −1** (agree): `Constants::init
  (00569a90)` lines 2398–2401 take `get_item`'s default, no
  `game/data/*.xml` names either tag (grep of all 65 files), and
  `crates/sim/src/territory/attrition.rs:430–435` already says so.

## Leaf `dow` (+0x270, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | runtime writer: `action_declare` `incl 0x270(%esi,%edi,4)` | 6dad3e, 6dad61 | same | agree | **confirmed**; net (2) finds no other | 6dad3e, 6dad61 |
| 2 | setup writers: `Leader::init` loop, `Leaders::clear` memset | line 243, `6ed5e0` | 6e3d44, memset | agree | **confirmed** | 6e3d44 (`0x19c(%esi)`, `esi=&good_deeds[j]`) |
| 3 | gate: counting block only when `quiet == 0` | yes | yes | agree | **confirmed** | 6dacf0 `jne 6dad68` |
| 4 | block A: `level == 0` and (`is_ally` or `invaders[t] < 5`) | yes | yes | agree | **confirmed** (signed `jge`) | 6dad14–6dad2c |
| 5 | block B: `is_ally(t)` still true before `set_diplo` | yes | yes | agree | **confirmed** | 6dad45–6dad4f |
| 6 | double count (A and B) for a forced level-0 on an ally | "counts 2" | unreachable | one-sided pair | **confirmed both**: the arithmetic gives 2, and no caller reaches it (the only `force = 1` caller, `DropControl`, passes `quiet = 1`) | 959758 / 959847 `pushl $1; pushl $1` |
| 7 | who passes `quiet = 0` | Declare; human hire party | adds a human `take_damage`, human nuke | **disagree** | **corrected (R2)**: only the Declare command and a human hire party (shared table) | 652277, 92c48b, 92c6ae |
| 8 | step `+1`, no decay, no scale | yes | yes | agree | **confirmed** | 6dad3e |
| 9 | reset: `Leader::init`, `Leaders::clear` only | yes | yes | agree | **confirmed**; `reset_score` does not touch it | 6e3d44 |
| 10 | reader 6bcf0a: own slot `dow[who]`, floored 1, divides `weak` | yes | yes | agree | **confirmed** (`imull $0x6ef0`, diagonal) | 6bcf03–6bcf18 |
| 11 | reader 6bd871: `leaders[t].dow[me] < 2` gate (human target) | yes | yes | agree | **confirmed** | 6bd871 |
| 12 | reader 6c0966 | own-slot diagonal, always true | `leaders[target].dow[me] < 2` | **disagree** | **corrected (R2)**: `imull $0x6ef0, %esi` then `0xe3a600(%eax)` is `leaders[esi].dow[esi]`, the diagonal | 6c0960–6c0966 |
| 13 | reader 6bfdcd: `leaders[t].dow[me] < 2/< 3` lifts the valuation by +100s | yes | yes (in `good_deeds`) | agree | **confirmed** | 6bfdc4–6bfe0b |
| 14 | reader 6c00cd: taunt pick on `!= 0` | yes | yes | agree | **confirmed** | 6c00cd |
| 15 | reader `LeaderData::log_data` | not named | yes | one-sided | **confirmed** (dump only) | 6e5f93 `pushl 0x1fc(%ebx)` |
| 16 | crate: no `action_declare`; `Sim::declare_war` `relations.rs:294`, `Sim::set_diplo` `:116`; SEAMs `diplomacy.rs:304`, `:398` | yes | yes | agree | **confirmed** (both functions exist at those lines; the SEAM comments name `dow`) | `crates/sim/src/leaders/relations.rs:116`, `:294` |
| 17 | plain game: human only; run1078 (style 0) shows `dow[1]` 0 → 1 | yes | yes, plus take_damage/nuke | agree, R2's extras **corrected** by #7 | **confirmed** (`crates/rondata/src/blind.rs:333–343`) — a capture note, not a diff assertion: no `rondata::diff` row compares `dow` | blind.rs:340 |
| 18 | locked style 1 without rush rules starts at war, so nothing reaches the counter | R1 | — | one-sided | **corrected (incomplete)**: enemies return at `is_enemy`, but a human's Declare on his own teammate (allied) passes, is turned into level 1, and counts once through block B. Whether the interface offers Declare on a locked teammate is **open** (read `IFaceDiplo*`'s Declare enabling, or one capture) | 6dab73–6dab8b, 6dad45 |
| 19 | age rush rules under style 1 keep the peace until someone declares, so a human Declare counts | R1 | — | one-sided | **confirmed in outline**: `action_declare` has no team-style test (6dab50–6dad86) and `end_the_peace` returns for `rush_rules <= 8` (`crates/sim/src/leaders/diplomacy.rs:75–77`) | 6dabf0 `war_allowed` |

Residual: a human Declare naming his own slot would also count the
diagonal (`is_ally(self)` holds, block B), so 6c0966's "always true" rests on
the interface never sending a self-target. No sim path passes one.

**Rule as a builder should port it.** Writers: a new
`Sim::action_declare(me, t, level, quiet, force)` beside `Sim::set_diplo`
(`crates/sim/src/leaders/relations.rs:116`), the human's Declare command and
the hire loop's human leg calling it with `quiet = 0`, every computer path
with `quiet = 1`. Predicate and step, after the refusals and before
`set_diplo`, only when `quiet == 0`: if `level == 0` and (`allied(me,t)` or
`invaders[me][t] < 5`): `if dow[me][t] != 0 { blacken[me] += 1 }; dow[me][t]
+= 1`; then if `allied(me,t)`: the same pair again. Integers, events, no
scale. Reset: 0 at `Leader::init` and `Leaders::clear`; never decremented.
Reached: only with a human declarer. An all-computer game keeps all 8×10
words at 0, which the crate's constant 0 already equals. **Confidence**:
high. Every site was read from the listing. **Open**: the interface's
Declare under locked teams (#18).

## Leaf `blacken` (+0x20c, int)

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | runtime writer `incl 0x20c(%esi)` in blocks A and B | 6dad38, 6dad5b | same | agree | **confirmed**; no other store | 6dad38, 6dad5b |
| 2 | setup writers: `Leader::init`, `Leaders::clear` | 6e3be0 | 6e3be0 | agree | **confirmed** | 6e3be0 |
| 3 | predicate: a counted `dow` event whose `dow[t]` was already non-zero (read before the increment) | yes | yes | agree | **confirmed** | 6dad2e–6dad36 |
| 4 | gate and callers as `dow` (`quiet == 0`, human only) | yes | yes + take_damage/nuke | agree, R2 extras **corrected** | as `dow` #7 | 652277, 92c48b |
| 5 | reader 6bd87e: `leaders[t].blacken < 4` (with `dow < 2`) lets the peace arm clear aggression | yes | yes | agree | **confirmed** | 6bd87e |
| 6 | reader 6bdc3b: race coin only while `blacken < 2` | yes | yes | agree | **confirmed**; the crate's site is `diplomacy.rs:444–451` | 6bdc3b |
| 7 | 6be667 | "×2 weight" (lines 1934/1958) | not named | one-sided | **corrected**: `cmpl $4, %edx; jge 6be9ae`, a `blacken < 4` gate into the demand arm (beside `hire_stamp`'s 13500 guard at 6be680) | 6be66d–6be670 |
| 8 | 6be7ec | "×2 weight" | not named | one-sided | **corrected**: the demand's size, `(25·epoch + 25)·(blacken + 1)` | 6be7e6–6be7f6 |
| 9 | ×2 cost term `(blacken != 0) + 1` | lines 1934/1958 (attributed to 6be667/6be7ec) | 6bf8d5, 6bf940 | **disagree on address** | **confirmed at R2's addresses**; R1's meaning was right and its addresses wrong | 6bf8d5, 6bf940 (`setne; incl; imull`) |
| 10 | 6bfcf4 | "tribute demanded scales by `(blacken+1)`" | the alliance `need`'s `(epoch + blacken + 1)` | **disagree** | **corrected (R1)**: the alliance-proposal `need = (my epoch + blacken[t] + 1) × (100, or 50 under `semaphore[0] & 4`) × ((*-0x48 != 0) + 1) − S'` | 6bfcd8–6bfd18 |
| 11 | 6c017e: message text on `blacken == 0` | yes | threshold 0 | agree | **confirmed** | 6c017e |
| 12 | 6c0b1a: `blacken >= 2` in the alliance logic | yes | threshold 2 | agree | **confirmed** | 6c0b1a |
| 13 | `consider_tribute` 6d1396: a computer receiver stamps only if amount `>= 100·leaders[giver].blacken` | yes | yes | agree | **confirmed** (`testb $4; jne` skips a human receiver first) | 6d138b–6d13a1 |
| 14 | `log_data` prints it | yes | yes | agree | **confirmed** | `log_data` line 619 |
| 15 | crate SEAMs `diplomacy.rs:398`, `:444` (`let blacken = 0`) | yes | yes | agree | **confirmed** | `crates/sim/src/leaders/diplomacy.rs:444–445` |
| 16 | `crates/rondata/src/diff/leader.rs:47` labels it "fog bookkeeping" | R1 | — | one-sided | **confirmed**: the label is wrong (a human's repeat-declaration count); listing it as unmodelled is right | leader.rs:47 |
| 17 | plain game: only a human's second declaration on a slot he already counted | yes | yes | agree | **confirmed** | 6dad36 |

**Rule.** Writers as `dow` (same new `Sim::action_declare`). Step: `+= 1`
inside each counting block when `dow[me][t] != 0` before that block's own
`+= 1`. It is a scalar summed over all targets, and a forced level-0 on an
ally could add 2 in one call, which no caller reaches. Reset at
`Leader::init` and `Leaders::clear`. Reached only by a human, on a second
counted declaration against the same slot. Readers to port with it (all
`leaders[t].blacken`): the gates `< 4` (6bd87e, 6be667), `< 2` (6bdc3b),
`>= 2` (6c0b1a) and `== 0` (6c017e, text only); the factors `(blacken + 1)`
in the demand size (6be7ec) and the alliance need (6bfcf4); `(blacken != 0) +
1` in two cost terms (6bf8d5, 6bf940); and `consider_tribute`'s `100·blacken`
floor. **Confidence**: high on writer and step; the reader list is
re-mapped by address here. **Open**: as `dow` #18.

## Leaf `broke_alliance` (+0x2b0, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | runtime writer `movl frame, 0x2b0(%esi,%edi,4)`, declarer's `[target]` only | 6dad7f | 6dad7f | agree | **confirmed**; the absolute range `0xe3a640..` has only reads | 6dad7f |
| 2 | setup writer in `Leader::init` | `puVar23[0x77]`, line 248 | "6e3d57 `0x1dc(%esi)`" | agree on the store | **confirmed at 6e3d5d** (R2's address is one instruction early: 6e3d57 is `0x1bc(%esi)` = `invaders[j]`) | 6e3d5d |
| 3 | R1's aside: 6e3d57/6e3d5d are `hire_stamp[2]`/`hire_who[2]` "after unrolling" | R1 | — | one-sided | **corrected**: off `esi = &good_deeds[j]` they are `invaders[j]` (+0x290) and `broke_alliance[j]` (+0x2b0) | 6e3b74, 6e3d8d |
| 4 | predicate: still allied both ways just before `set_diplo`, **any `quiet`** | yes | yes | agree | **confirmed** (`quiet != 0` jumps to 6dad68, the stamp's own test) | 6dacf0 → 6dad68–6dad72 |
| 5 | the target's mirror slot not written | yes | yes | agree | **confirmed** | 6dad7f only |
| 6 | live breaking callers include `diplomacy` 6beceb | yes (line 1501) | yes | agree | **corrected**: 6beceb is dead (shared table). The live explicit break is 6bed1c, plus any level-0 call whose target is allied both ways (turned into level 1), plus `action_respond` 6d04ed, the human Declare, `DropControl` | 6becdb, 6bca78 |
| 7 | readers: `action_declare`'s refusal; `Unit::process_attrition (005e11a0)` lines 164–168 | yes | yes | agree | **confirmed** | 6dab8d–6dabc5; 5e161c/5e162c/5e164b |
| 8 | interface readers `Options::do_declare_war (007216b0)` lines 223–232, `IFaceMainBase::draw_timers (0080b060)` lines 761–775 | R1 | — | one-sided | **confirmed** (both compare against `ally_to_war_delay`) | do_declare_war:223–231 |
| 9 | inert on shipped data (delay −1) | yes | yes | agree | **confirmed** | shared |
| 10 | in the checksum list | — | `checksum/leaders.rs:81` | one-sided | **confirmed** | `crates/sim/src/checksum/leaders.rs:81` |
| 11 | crate: "rides `Sim::set_diplo` level 1" (R2) vs "`set_diplo` level 1 is the console's peace verb, stamp belongs to the declare path only" (R1) | | | **disagree in wording** | **R1 confirmed**: `set_diplo`'s `_` arm (`relations.rs:133–138`) is also what an accepted treaty would call, and the original stamps only in `action_declare`, so the stamp goes in the new `action_declare`, never in `set_diplo`. R2 says the same in its next clause | `relations.rs:116–139` |
| 12 | plain game: any alliance break; style 1 reachability not read | both | both | agree | **open, narrowed**: 6bed1c sits in the demand arm, reachable only via 6be9ab/6be9ae (jumped to from 6be65e–6be6ca), which the sibling's §(A) shows gated off at style 1 (`6be556`). The level-0 sites 6be1ef/6be210 (a not-at-war target, rush rules) and 6c04df were not traced to a style-1 verdict; whether one can name a mutual ally decides it (a capture with a team and rush rules, or reading 6be067–6be1ad's entry) | 6be556; 6be18e–6be1ab |

**Rule.** Writer: in the new `Sim::action_declare`, after the counting
block and for any `quiet`: if `allied(me, t)` (both ways 2) then
`broke_alliance[me][t] = frame` (the sim frame), before `set_diplo(me, t,
level)`. Reset 0 at `Leader::init`/`Leaders::clear`. Readers compare `frame
− max(mine[t], theirs[me])` with `ALLY_TO_WAR_DELAY`, which is −1 in shipped
data, so no outcome depends on it. It is still a dump and checksum word, so
the port must write it. Crate site: none today. The reader input is
`attrition.rs`'s `in_war_grace`. Reached: by any alliance break that goes
through `action_declare`. The crate's `end_the_peace` and console verb go
through `set_diplo` directly in the original too (`Game::do_frame`'s
`set_diplo(i,j,0)` skips allies), so they must not stamp. **Confidence**:
high on the store. **Open**: whether an all-computer style-1 game ever
breaks an alliance (#12).

## Leaf `made_peace` (+0x2d0, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | writers: `action_respond` stamps both sides, `responder.made_peace[proposer]` and `proposer.made_peace[responder]` | 6d108d, 6d10a0 | same | agree | **confirmed**; no other store | 6d108d, 6d10a0 |
| 2 | setup: `Leader::init` loop, `Leaders::clear` | line 249 | 6e3d63 | agree | **confirmed** | 6e3d63 |
| 3 | treaty word is `responder.dip[proposer].treaty` (`+0x6934`) | yes | yes | agree | **confirmed** | 6d100c `0xe40cc4(%ecx,%esi)` |
| 4 | `-1` skips the treaty block (straight to the hire loop) | yes | yes | agree | **confirmed** | 6d1016–6d1019 `je 6d11bc` |
| 5 | both directions set to `max(treaty, current diplos)`, signed | yes | yes | agree | **confirmed** | 6d1029–6d1032, 6d104d–6d1066 |
| 6 | stamp iff treaty word `== 1`, even if the `max` leaves the pair allied | yes | yes | agree | **confirmed** | 6d1072 `cmpl $1,-0x24(%ebp)` |
| 7 | gating into the apply arm: `response == 1` and the `agree` words | outline | outline | agree | **confirmed in outline**; not re-derived here (the sibling's §(A) walks every route into `action_respond`) | — |
| 8 | reader `Unit::process_attrition` lines 171–174 only; inert (delay −1) | yes | yes | agree | **confirmed** (net (1): reads 5e1697, 5e16d0 only) | 5e1697, 5e16d0 |
| 9 | callers of `action_respond(.., 1)` | `diplomacy`, `process_accept`, `ScenarioFuncSet` | adds `process_taunt` 6b8f16, `action_agree` | agree | **confirmed** (the sibling's §(A) reads the same list) | — |
| 10 | crate: no `Diplomacy` records, no `action_respond`; SEAM `diplomacy.rs:177` / `:480–495` | yes | yes | agree | **confirmed** | `crates/sim/src/leaders/diplomacy.rs:177` |
| 11 | plain game: an accepted peace treaty; style 1 not read | both | both | agree | **settled for all-computer style 1: unreachable**. No `action_respond` is ever called there (sibling §(A), deciding lines `6bdf56`, `6c0a39`, `6c09c8`, `6be556`). Reachable with a human (`process_accept`) or under styles 0/8/11 | sibling §(A) |

**Rule.** Writer: a new `Sim::action_respond(responder, proposer,
response)`. In its apply arm, after the goods exchange, let `w =
responder.dip[proposer].treaty`. If `w != -1`:
`set_diplo(responder, proposer, max(w, diplos[responder][proposer]))`, then
`set_diplo(proposer, responder, max(w, diplos[proposer][responder]))`, and if
`w == 1` set both `made_peace` words to the sim frame. Reset 0 at
`Leader::init`/`Leaders::clear`. The only reader is attrition's
`ALLY_TO_WAR_DELAY` window, which is inert with shipped data. Dump and
checksum word only. Reached: with a human proposal or answer, or an unlocked
team style. Never in an all-computer style-1 game. **Confidence**: high.
**Open**: nothing for the store. Which offers carry `treaty == 1`
(`action_treaty`, `action_offer`) belongs to the family-8 port.

## Leaves `hire_stamp` (+0x1b4, int[8]) and `hire_who` (+0x1d4, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | four stores in `action_respond`'s hire loop: `responder.hire_stamp[X]` (abs), `proposer.hire_stamp[X]` (`-0x20(%ecx)`), `responder.hire_who[X] = proposer` (abs), `proposer.hire_who[X] = responder` (`(%ecx)`) | 6d121e, 6d122e, 6d123a, 6d1244 | same | agree | **confirmed**; `ecx` is `&leaders[proposer].hire_who[X]` from `6d11d4 leal 0xe3a564(%eax)` with `eax = -0x14(%ebp)`, stepped `+4` at 6d12a0 | 6d11d4–6d1244 |
| 2 | setup: `Leader::init` `hire_stamp[j]=0`, `hire_who[j]=-1`; `Leaders::clear` 0/0 | yes | 6e3c2d, 6e3c36 | agree | **confirmed**; and every slot runs `Leader::init` from `Leaders::init` | 6e3c2d, 6e3c33–6e3c36, 6ed8cb |
| 3 | gate: `Game::war_allowed` | yes | yes | agree | **confirmed** | 6d11c2–6d11c9 |
| 4 | per X in 0..8: `X != responder`, `X != proposer`, `responder.dip[proposer].attacks[X] != 0` (`+0x6968`) | yes | yes | agree | **confirmed** | 6d11e0–6d1206 |
| 5 | stamps precede the declarations and do not depend on them | yes | yes | agree | **confirmed** | 6d121e < 6d1273 |
| 6 | each party not already an enemy of X declares (level 0, `quiet = !human`, force 0) | yes | yes | agree | **confirmed** | 6d1258–6d1297 |
| 7 | `attacks[]` written by `Leader::action_attack (006d14e0)` into both parties' records; callers `diplomacy` 6c0f2b, `CommandPackage` 946c32, `ScenarioFuncSet` 9fc373; `Leader::action_propose_attack (006d14d0)` is a `jmp` to it | R1 | — | one-sided | **confirmed** (`action_attack` lines 12, 14; the three `calll 0x6d14e0` and the `jmp`) | 6d14d0, 6c0f2b, 946c32, 9fc373 |
| 8 | the computer's alliance-plus-attack offer is skipped outside styles 0/8/11 | R1 | — | one-sided | **confirmed** (sibling §(A), `6c09b6`–`6c09c8`) | 6c09c8 |
| 9 | reader 6bdc8a: `me.hire_stamp[t] != 0 && frame − it < 13500` → `aggression[t] = 1` | yes | yes | agree | **confirmed** | 6bdc8a–6bdca9 |
| 10 | reader 6bdce2: the mine coin clears aggression only if `hire_stamp[t] == 0` or `frame − it > 4500`; coin drawn first | yes | yes | agree | **confirmed** | 6bdcd3, 6bdce2–6bdd04 |
| 11 | loop arm at 6bdd57 | **dead**: compares `leaders[l].hire_who[me]` with the loop index `l` | "a third leader's `hire_who[target] == candidate`" forces aggression | **disagree** | **R1 confirmed, R2 corrected**: `edx = l·0x1bbb + me` (6bdd51), `cmpl %edi, 0xe3a564(,%edx,4)` with `edi = l` (6bdd12, 6bddf8). It reads `[me]`, not `[target]`, against `l` itself. A writer stores the *other* party, and init stores −1 in every slot, so the test needs a deal whose responder is also its proposer. No caller makes one | 6bdd51–6bdd77 |
| 12 | loop arm (b): `t` my ally, `me.hire_who[l] == t`, `frame − me.hire_stamp[l] <= 9000`, I am at war with `l`, `t` is not → `aggression[t] = 1` | R1 (full) | names `hire_who` "off `&field_0x1d4`" only | one-sided detail | **confirmed** (`esi = &me.hire_who[l]`; `-0x160(%esi)` = `me.diplos[l]`; `jg` skips above 9000; `is_ally` at 6bdddb) | 6bdd20, 6bdd87–6bddea |
| 13 | reader 6bf7a5 `-0x1dc(%esi)` is `hire_who` | — | R2 | one-sided | **corrected**: there `esi = me + 0x250 + 4k` (6bf6d3), so `-0x1dc(%esi)` is `me.diplos[k]` (+0x74), compared with 2. Not a hire reader | 6bf6d3, 6bf7a5–6bf7ad |
| 14 | reader 6be680: `hire_stamp[t] != 0 && frame − it <= 13500` diverts the demand arm | "13501" | "13501 (`0x34bd`)" | agree in effect | **confirmed** with the listing's constant: `cmpl $0x34bc; jle 6be9ae` (≤ 13500 ≡ < 13501) | 6be68d–6be692 |
| 15 | readers live: three (R1) vs the crate SEAM's "two" | R1 | — | one-sided | **confirmed**: at the crate site (`diplomacy.rs:453`) three arms read it (#9, #10, #12). A fourth read (#14) sits in the demand arm, outside that site | `crates/sim/src/leaders/diplomacy.rs:453–454` |
| 16 | `BuildOut::make_options`' `hire_who[4]` is a mistyped `UnitType*` read (`+0x1e4`) | yes | yes | agree | **confirmed** (net (1) has no such absolute; read only) | — |
| 17 | crate: no hire model; SEAM `diplomacy.rs:453`; a new `action_respond` | yes | yes | agree | **confirmed** | as #15 |
| 18 | plain game: needs an executed deal with an attack clause, 3+ leaders, war allowed | yes | yes | agree | **confirmed**, and **unreachable in an all-computer style-1 game** (no `action_respond`, sibling §(A)) | sibling §(A) |

**Rule.** Writer: the new `Sim::action_respond`. At the end of the apply
arm (after the treaty block, reached also when `treaty == -1`), if
`war_allowed`, then for each X in 0..8 with X ∉ {responder, proposer} and
`responder.dip[proposer].attacks[X] != 0`:
`hire_stamp[responder][X] = hire_stamp[proposer][X] = frame`,
`hire_who[responder][X] = proposer`, `hire_who[proposer][X] = responder`, and
then responder, then proposer, if not an enemy of X,
`action_declare(X, 0, quiet = !human, 0)`. A human party's declaration
counts `dow`/`blacken`. Reset: `hire_stamp` 0, `hire_who` −1 at
`Leader::init`. `Leaders::clear` leaves 0 in both until then. Readers to
port: #9, #10, #12, #14. #11 is a test that cannot pass, and porting it as
written is exact. Crate site: `diplomacy.rs:453` for the readers, a new
`action_respond` for the writers. Reached: only with an executed attack deal.
Never in an all-computer style-1 game, where every slot stays `0`/`−1`.
**Confidence**: high.

## Leaf `good_deeds` (+0xd4, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | sole runtime writer `subl %ebx, 0xd4(%edi,%esi,4)` in `Leader::diplomacy` | 6c08ae | 6c08ae | agree | **confirmed** (net (2); the absolute range `0xe3a464..` has no access) | 6c08ae |
| 2 | setup: `Leader::init` `(%esi)` at 6e3c10; `Leaders::clear` | line 182 | 6e3c10 | agree | **confirmed** | 6e3c10 |
| 3 | `plan_strategy`'s `movl 0xd4(%ecx)` and the `0xdc..0xe8` stores (encrypted block) are other bases | yes | yes | agree | **confirmed** | 6ba8c2; 6d298f |
| 4 | the valuation's score is seeded with `good_deeds[t]` | yes | yes | agree | **confirmed**; every later write to the local accumulates (`subl`/`addl`, or `movl -0x2c → edx; add; movl edx → -0x2c`) | 6bf14a–6bf151 |
| 5 | non-alliance: `need = −S`; alliance (`treaty == 2`): `need = (my epoch + blacken[t] + 1)·(100 \| 50)·((*-0x48 != 0) + 1) − S'`, with `S' = S + epoch[t]·{0,50,100}` and adjustments | `need = −S` only | both | one-sided (alliance) | **confirmed** (R2); the alliance path jumps past the `negl` to 6bfe19 | 6bfe17; 6bfc32–6bfd1d |
| 6 | `need −= 99` when `counteroffer[t] != 0` | yes | yes | agree | **confirmed** (`leal -0x63(%ecx)` + `cmove`) | 6bfe1c–6bfe2a |
| 7 | early-game multiplier on asked goods: ×4 before frame 4500; ×3/2 (toward zero) before 9000 when `*(-0x40) == 0` | — | R2 (names it `*local_44`) | one-sided | **confirmed** with the pointer named `-0x40(%ebp)`, not `-0x44`. `-0x44` is the "something asked" flag set at 6bf533 | 6bf549–6bf571 |
| 8 | `-0x50` is "asks more than I hold" | unread | named from use | one-sided | **confirmed**: `cmovg` sets it when the asked amount exceeds my decrypted stock (`^0x8221`) | 6bf51b–6bf52c |
| 9 | accept branch: `-0x50 == 0` and `need <= 0` | yes | yes | agree | **confirmed** | 6bfe30–6bfe32, 6bfff4 |
| 10 | at accept: `counteroffer[t] = 0`; **only if `any_proposals(t)`**: `action_respond(t, 1)`, then `need += 50` if `weak[t] != 0` and `!is_tribute_period(leaders[t], me)`, then `good_deeds[t] −= need` | yes | yes | agree | **confirmed**; without a pending proposal the arm returns before the store | 6c086b–6c08ae (`6c087d je 6be98d`) |
| 11 | the old balance enters twice: `good_deeds' = good_deeds + S (+99 if a counteroffer was pending) (−50 if weak, outside a tribute period)` with `S` seeded by `good_deeds` | implicit ("grows by the surplus") | explicit, flagged as surprising | agree | **confirmed**: the listing doubles the balance on every accepted deal. The increment is `−need ≥ 0` less the 50, so it can fall by at most 50 | 6bf14a + 6c08ae |
| 12 | sole reader: the next valuation's seed | yes | yes | agree | **confirmed** for the sim | 6bf14a |
| 13 | not logged by `LeaderData::log_data` | R1 | — | one-sided | **corrected**: `log_data` prints it per slot, `pushl 0x60(%ebx)` with `ebx = &diplos[j]` | 6e588e |
| 14 | CtW/scenario fixed-answer path skips the valuation (`semaphore[2] & 2` → `0xcc21e0` mode) | R1 | — | one-sided | **confirmed** for the `semaphore[2] & 2` half (`testb $2, 0x822`, then the 1/2/3 switch). The `semaphore[1] & 0x10` half was not re-read | 6bf118–6bf13d |
| 15 | crate: inside the offers-arm SEAM, `diplomacy.rs:490` (`LAB_006bf0e0`); `diplomacy_with` at `:174` | yes | yes | agree | **confirmed** | `crates/sim/src/leaders/diplomacy.rs:174`, `:490` |
| 16 | plain game: a computer accepting a pending proposal | yes | yes | agree | **confirmed**, and **unreachable in an all-computer style-1 game**: `any_proposals` is never true there (sibling §(A)) | 6c087d |

**Rule.** Writer: inside the ported offers arm of
`Sim::diplomacy_with` (`crates/sim/src/leaders/diplomacy.rs:174`, at the
`:490` SEAM). Seed `S = good_deeds[me][t]` and accumulate the valuation into
it, in the valuation's own integer units (resource value × amount >> 8, flat
25/50/100 steps). Then `need = −S`, or the alliance form for `treaty == 2`;
`need −= 99` if `counteroffer[me][t] != 0`. On `need <= 0` with the
over-ask flag clear: `counteroffer[me][t] = 0`; if `any_proposals(t)`:
`action_respond(t, 1)`, `if weak[me][t] != 0 && !is_tribute_period(t, me) {
need += 50 }`, `good_deeds[me][t] −= need`. Reset 0 at `Leader::init` and
`Leaders::clear`. No decay. Reached: only when a computer accepts a pending
proposal (a human's, or under styles 0/8/11). **Confidence**: high on the
store, the gate and the seed. Medium on the full composition of `S`, which
neither reading walked term by term. That walk belongs to the offers-arm
port.

## Leaf `last_spoke` (+0x2f4, int[8])

| # | claim | R1 | R2 | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1 | no runtime writer | yes | yes | agree | **confirmed**: no absolute in `0xe3a684..0xe3a6a3` anywhere; in leader-class code the only `0x2f4..0x313` hits are `movw` 16-bit stores off other bases (6b9bdc, 6e4b3b), `scale_tribute`'s `Constants` words, `calc_gather`/`gain_tech`/`calc_rare` off non-leader bases, and `get_support_count`'s `testb $1, 0x2f4(%eax)` off `[0xc0a274][who]`, a type table | 6da131–6da139 |
| 2 | setup: `Leader::init` 6e3d6f; `Leaders::clear` | line 251 | 6e3d6f | agree | **confirmed** | 6e3d6f |
| 3 | no `$0x2f4`/`lea` forming its address | yes | yes | agree | **confirmed**: the only `addl $0x2f4` immediates are four `__unwindfunclet` thunks of `HelpBox`/`GameSpy` | — |
| 4 | the eight `0xe3a680(%ecx)` stores in `diplomacy` are `got_diplo_message` (+0x2f0, scalar) of `leaders[t]` | yes | yes | agree | **confirmed** (no index register; `Leader::init` zeroes it as a scalar at 6e3d69 `0x2f0(%ebx)`) | 6becc8, 6e3d69 |
| 5 | readers: none in sim code | yes | yes | agree | **confirmed**; `log_data` prints it (`pushl 0x280(%ebx)`, 6e6269) | 6e6269 |
| 6 | save/scenario paths (`walk_data`, `load_leaders_chunk`) could copy a value in | open (R1) | `walk_data` restores | agree | **confirmed as a copy only**: neither originates a value, and a save written by the original holds 0 | — |
| 7 | crate: constant 0; checksum row `checksum/leaders.rs:84` | yes | yes | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:84` |

**Rule.** No event. Carry it as 0 for the dump and the checksum walk.
Reset 0 at `Leader::init`/`Leaders::clear`. Every game "reaches" it as a
constant. **Confidence**: high. **Open**: none.

## Side findings (outside the eight stores)

- **`crates/sim/src/combat/take_damage_view.rs:34` `LEADER_OUT = 0x4`, "the
  player is out"** (R1, one-sided): **confirmed misnamed**. Bit 4 of
  `leader_flags` is HUMAN (`docs/ARMY.md` line 103, the PDB enum), and the
  body at `6ec170` returns `flags & 4`. The arithmetic at its uses
  (`take_damage_view.rs:192`, `:356`, `:374`) is unchanged by the name, but
  each reads "skip a human declarer", which is the predicate that keeps
  `take_damage` from ever passing `quiet = 0` here.
- **`crates/sim/src/leaders/diplomacy.rs:453–454`** says "neither of their two
  arms fires". Three arms at that site read the hire words (hire leaf #15).
- **`crates/rondata/src/diff/leader.rs:47`** labels `blacken` "fog
  bookkeeping" (blacken leaf #16).
- **README row 7** ("the `Leader::action_*` family and `Leader::diplomacy`'s
  arms"). Settled writers: `Leader::action_declare (006dab50)` for `dow`, `blacken`
  and `broke_alliance`; `Leader::action_respond (006d03c0)` for `made_peace`,
  `hire_stamp` and `hire_who`; `Leader::diplomacy (006bc950)` for
  `good_deeds` alone. `last_spoke` has none. Every one of them also gets
  `Leader::init (006e3930)`'s loop and `Leaders::clear`'s memset.

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| `dow` | 16 / 3 / 1 | `action_declare` with `quiet == 0` (a human's Declare, or a human hire party): `+1` to `dow[me][t]` for a level-0 war on a non-ally with `invaders[me][t] < 5`, and `+1` for a break of an alliance; 0 at init; all-computer games keep 0 |
| `blacken` | 13 / 4 / 0 | `+1` whenever one of `dow`'s counted steps finds `dow[me][t]` already non-zero (same gate); readers re-mapped: gates `<4`, `<2`, `>=2`, `==0`, factors `(blacken+1)` ×2 sites, `(blacken!=0)+1` ×2 sites, `consider_tribute`'s `100·blacken` |
| `broke_alliance` | 9 / 2 / 1 | `action_declare`, any `quiet`: declarer's `[t] = frame` when the pair is still allied both ways before `set_diplo`; inert with shipped `ALLY_TO_WAR_DELAY = −1`; 6beceb is dead |
| `made_peace` | 11 / 0 / 0 | `action_respond` apply arm: if `responder.dip[proposer].treaty == 1`, both sides' `[other] = frame` after the `max` `set_diplo` pair; inert reader; never in all-computer style 1 |
| `hire_stamp` / `hire_who` | 16 / 2 / 0 | `action_respond` apply arm, if `war_allowed`: for each attack clause X ∉ {both parties}, both `hire_stamp[X] = frame` and each `hire_who[X] =` the other party, then the declarations; init 0 / −1; loop arm 6bdd57 dead as compiled |
| `good_deeds` | 15 / 1 / 0 | `Leader::diplomacy` accept branch, only if `any_proposals(t)`: `good_deeds[me][t] −= need` (+50 when weak outside a tribute period), where `need` is built from a score seeded with `good_deeds[me][t]`, so the balance doubles per accepted deal |
| `last_spoke` | 7 / 0 / 0 | no writer but init/clear: constant 0, dump and checksum only |
| **total** | **87 / 12 / 2** | |

The two opens are both reachability questions. Neither changes a rule:
(1) whether the interface offers Declare on a locked teammate (`dow` #18;
the answer also decides `blacken` and `broke_alliance` for a human), and
(2) whether an all-computer style-1 game ever breaks an alliance through
6be1ef/6be210/6c04df (`broke_alliance` #12).
