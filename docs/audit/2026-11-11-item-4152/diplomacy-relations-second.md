# diplomacy-relations: blind second reading (item 4152, wave 2)

- Slug: `diplomacy-relations`
- Leaves (LeaderData offsets): `good_deeds` +0xd4 int[8], `hire_stamp` +0x1b4 int[8], `hire_who` +0x1d4 int[8], `blacken` +0x20c int, `dow` +0x270 int[8], `broke_alliance` +0x2b0 int[8], `made_peace` +0x2d0 int[8], `last_spoke` +0x2f4 int[8]
- Reader model: Claude Opus 5.5 (claude-opus-5-5)
- Sources: decompile export `~/ghidra-projects/decomp`, PE listing (llvm-objdump of `riseofnations.exe` .text), `crates/sim`. Blind: nothing under this audit directory but this file was opened.

## Ground rules established first

- **Base register.** In the listing, a `Leader*` (`this` of `Leader::*`) points at the `LeaderData` base: `Leader::diplomacy (006bc950)` reads `hire_stamp` as `0x1b4(%ebx,%eax,4)` and `blacken` as `0x20c(%edx)`, and `Leaders::clear (006ed5d0)` memsets from `0xe3a390` per slot. So the listing displacements are the `LeaderData` offsets, unshifted; the global form is `0xe3a390 + slot*0x6eec + off`, and the pair form `slot*0x1bbb + other` indexes `[.. ,4]`. The decompile's `Leader` type is wrong in stride (it prints `*0x6ef0`, `*0x6f48`) but its `field_0xNNN` names on a `Leader*` match the `LeaderData` offsets; I used the listing for every address.
- **Frame.** `GameAccess::game` is `[0xc061ec]`, and `game->frame` is `+0x550`. `GameAccess::constants` is `[0xc061f0]`; `ally_to_war_delay` is its `+0xcf8`.
- **Diplos.** `LeaderData::is_enemy (006ebaa0)`: different slots and `diplos` 0 either way. `LeaderData::is_ally (006edb50)`: same slot, or `diplos == 2` both ways. So 0 war, 1 peace, 2 allied.
- **`leader_flags & 4` is HUMAN** (`docs/ARMY.md` line 103, the PDB enum).
- **Nets.** Export: `->leaf` greps return only `hire_who` (in `BuildOut::make_options`, a misnamed read through an unrelated pointer) and `blacken` (a read in `LeaderData::log_data`); the decompile prints every other access as `field_0xNNN`. Listing: a scan of the whole `.text` for every write-shaped instruction (`mov/add/sub/inc/dec/and/or/xor/neg/not/shift/set*/xchg/rep stos`) whose destination is one of the leaves' displacements (`off + 4k`) or an absolute in `[0xe3a390, +10*0x6eec)` that reduces to one, plus every `lea`/`mov $imm`/`push $imm` that forms a leaf's address. Each hit in a non-`Leader` function was checked for its base register; the false positives are listed under each leaf.
- **Block writers that touch every leaf** (stated once here): `Leaders::clear (006ed5d0)` memsets `LeaderData[0 .. 0x6929)` to 0 for all 10 slots (`push 0x6929; lea -0x6c14(%esi)`, `esi` starting `0xe40fa4`); `LeaderData::walk_data (006d6750)` walks the byte range `[who, field_0x692a)` as one block when `leader_flags & 1` (save/load and the sync walk); `Game::run_oos_recovery` and `Game::close` hit the scan only at displacements on other bases (not read further, see confidence).

## `action_declare`'s arguments, read at every call site

`Leader::action_declare (006dab50)` is `__thiscall (this = declarer, target, level, quiet, force)`; the listing names them `0x8(%ebp)` .. `0x14(%ebp)`. Arguments pushed right to left, read from the two to six instructions before each `call 0x6dab50` (16 sites):

| caller | target | level | quiet (`+0x10`) | force (`+0x14`) |
|---|---|---|---|---|
| `Object::take_damage` (6522ed) | `ebx` | 0 | `~(flags >> 2) & 1` of the declarer, i.e. 1 unless HUMAN | 0 (`eax` = the 0 `is_enemy` returned) |
| `Leader::diplomacy` 6be1ef, 6be210, 6bed2f, 6c04df | `esi` | 0 | 1 | 0 |
| `Leader::diplomacy` 6beceb, 6bed1c | `esi` | 1 | 1 | 0 |
| `Leader::action_respond` 6d04ed (the refuse-2 arm: the proposer breaks with a human responder) | responder | 1 | 1 | 0 |
| `Leader::action_respond` 6d1273, 6d1297 (the hire loop) | X | 0 | `~(flags >> 2) & 1` | 0 |
| `Nuke::do_damage` 92c502, 92c71e | | 0 | `~(flags >> 2) & 1` | 0 |
| `CommandPackage::process_declare` 9473aa | packet | packet | **0** | 0 |
| `DropControl::process_drop` 959761, 959850 | `edi` | 0 | 1 | **1** |
| `ScenarioFuncSet::declare_war` 9fbedb | `esi` | 0 | 1 | 0 |

`CommandPackage::process_declare` is reached only from `CommandManager::issue_declare (00942200)`, whose callers are `LeaderOut::issue_declare (006d4080)` (from `Options::exec`) and `Popup::on_modal_end` — the interface. So **quiet = 0 only for the interface's declare and for a HUMAN declarer's implicit declares** (take_damage, nuke, a hire). Every computer declaration passes quiet = 1.

The body, in my words (listing 6dab50–6dadb0):

1. `is_enemy(target)` → return 0.
2. Top of a loop: if `level == 0 && force == 0` and `is_ally(target)`: set `level = 1` and go round again (a war on an ally becomes a break to peace). If `level == 0 && force == 0` and not allied: `t = max(this.broke_alliance[target], leaders[target].broke_alliance[this])`; if `t != 0 && frame - t < ally_to_war_delay` → refuse (message to a console declarer), return 1.
3. If `quiet == 0` and `!LeaderData::afford_dow (006d5ce0)` → `LeaderOut::warn_resources`, return 1.
4. If `level == 0` and `!Game::war_allowed (00594670)`: with `force == 0` → `say_no_war`, return 1; with `force != 0` → `level = 1`, round again.
5. **If `quiet == 0`**: `Leader::pay_dow (006d2b10)(target, level)` (and a second `pay_dow(target, 0)` when allied and `level == 0`); then the two counting blocks of `dow`/`blacken` below.
6. **Whatever `quiet`**: if `is_ally(target)` still holds (before the relation changes) → `broke_alliance[target] = frame`.
7. `Leader::set_diplo (006ec6a0)(target, level)`, then the team walk (`ally_diplo` for teammates of either side), then the messages.

## `dow` (+0x270, int[8])

**1. Writers.**
- `Leader::action_declare (006dab50)`: `incl 0x270(%esi,%edi,4)` at 6dad3e and again at 6dad61 (`esi` = declarer, `edi` = target). The only game-state writer.
- `Leader::init (006e3930)`: `movl %eax, 0x19c(%esi)` at 6e3d44 in the per-other loop (`esi = &good_deeds[j]`, so `0xd4 + 0x19c = 0x270`), `eax = 0`, for j in 0..8.
- Block writers: `Leaders::clear (006ed5d0)` (0), `LeaderData::walk_data (006d6750)` (restore).
- False positives checked: `Leaders::clear` 6ed7cf–6ed7ed (BitMask clears off `esi` deep in the record, not `LeaderData+0x270`), `ConsoleWin::run_cmd` 7d7983 and `ScenarioRead::import` 9b2c2d (base `[0xe335a8]`, not a leader), `ScenarioRead::load_leaders_chunk` 9acf5b.. (base tests `0x821(%edx)`, the game info record; String-shaped `movl/movl/movw` runs), `IFaceEditorDiplo`, `OptionsWinGame::do_load`, `ScriptEditBox` (interface records).

**2. Event, step, predicate.** The declarer's `dow[target] += 1` per declaration, a count. Only when `quiet == 0` (the human paths of the table above). Two blocks, both before `set_diplo`:
- Block A (6dad14): only `level == 0`; skipped when the target is not allied **and** `invaders[target]` (`+0x290`) `>= 5`. Then: if `dow[target] != 0` → `blacken += 1`; `dow[target] += 1`.
- Block B (6dad45): when `is_ally(target)` (current relation, both ways allied): the same pair of steps.

Reachable combinations with `quiet == 0` (every such caller passes `force == 0`, so a level-0 war on an ally is first turned into level 1): a war on a non-ally (`level 0`) counts once unless `invaders[target] >= 5`; a break of an alliance (`level 1`, from allied) counts once through block B; a `level 1` on a leader merely at peace counts nothing. The double count (A and B for one call) needs `level 0` on an ally with `force != 0`, which only `DropControl` passes, and it passes `quiet = 1`: unreachable.

**3. Reset.** `Leader::init` zeroes all 8 slots; `Leaders::clear` zeroes the record. Nothing decrements it. Never written by the computer's own declarations.

**4. Readers.** `Leader::diplomacy (006bc950)`: its own slot `dow[who]` floored at 1 as a divisor (decompile line 346, listing 6bcf0a `0xe3a600(%eax)`); `leaders[target].dow[me] < 2` gates arms of the peace/war choice (lines 775, 2134, 2702, listing 6bd871, 6c0966); `!= 0` picks a taunt (line 2828, 6c00cd). `LeaderData::log_data`.

**5. Crate.** No `action_declare` exists. The crate's declare is `Sim::declare_war` (`crates/sim/src/leaders/relations.rs:294`, reached by `Command::declare_war`, `crates/sim/src/command.rs:370`) and `Sim::set_diplo` (`relations.rs:116`), both at `set_diplo` depth. `crates/sim/src/leaders/diplomacy.rs:304` and `:398` carry `dow` as `SEAM: dow is not carried`. A builder adds the counter in a new `action_declare` around `declare_war`, keyed on `quiet == 0` (a human declarer), with `invaders` (also not carried) for block A's exemption.

**6. Plain game?** Yes, but only a **human** declarer steps it: the diplomacy window's Declare (run1078 in `crates/rondata/src/blind.rs:333–343` records `dow[1]` 0 → 1 on block 912 after `CM process_declare`, which is this path), a human unit damaging a non-enemy (`Object::take_damage`), a human nuke, and a human accepting a hire. A computer-only game never moves it. `DropControl` is multiplayer; `ScenarioFuncSet::declare_war` is the trigger layer (quiet 1 anyway).

**7. Confidence.** High on the step and predicate (listing read instruction by instruction, every call site's arguments read from the listing). Not read: `pay_dow`'s body beyond confirming it writes none of the leaves (the scan saw no hit in it); `Game::run_oos_recovery`/`Game::close` bodies.

## `blacken` (+0x20c, int)

**1. Writers.**
- `Leader::action_declare (006dab50)`: `incl 0x20c(%esi)` at 6dad38 (block A) and 6dad5b (block B).
- `Leader::init (006e3930)`: `movl %ecx, 0x20c(%ebx)` at 6e3be0, `ecx = 0`.
- Block writers: `Leaders::clear`, `LeaderData::walk_data`.
- False positives checked: `Objects::init` 65f2db/65f329 (`edi` an `Objects`/type record; the decompile's own fields), `ScenarioRead::load_leaders_chunk` 9acf98 (game-info base), `ConquestGame::close` (Conquest record).

**2. Event, step, predicate.** `blacken += 1` on a declaration that steps `dow[target]` while `dow[target]` is already non-zero — a repeat declaration on the same target, counted once per repeat, across all targets. The test reads `dow[target]` before its own increment. Same `quiet == 0` gate and block A / block B predicates as `dow` (above). A plain count, no scale.

**3. Reset.** `Leader::init` → 0; `Leaders::clear`. Never decremented.

**4. Readers.** `Leader::diplomacy`: `cmpl $4 / $2 / $0, 0xe3a59c(..)` (6bd87e, 6bdc3b, 6c017e, 6c0b1a: `leaders[target].blacken` thresholds 4, 2, 0), and `(blacken != 0) + 1` doubles a cost term (decompile lines 1934, 1958; listing 6bf8d5, 6bf940 compare `0x20c(%edx|%edi)`). `Leader::consider_tribute (006d1360)`: a tribute from leader p at least `leaders[p].blacken * 100` (6d1396 `imull $0x64`) can stamp `tribute_stamp` — the 100 is in the tribute's own resource units. `LeaderData::log_data` prints it.

**5. Crate.** Not carried: `crates/sim/src/leaders/diplomacy.rs:398` and `:444–446` (`let blacken = 0;  // SEAM`). Same builder site as `dow`: a new `action_declare` wrapping `Sim::declare_war` (`crates/sim/src/leaders/relations.rs:294`).

**6. Plain game?** Only through a human declaring on the same leader a second time (or breaking an alliance with a leader already declared on). Not reachable in a computer-only game.

**7. Confidence.** High. Same unread list as `dow`.

## `broke_alliance` (+0x2b0, int[8])

**1. Writers.**
- `Leader::action_declare (006dab50)`: `movl %eax, 0x2b0(%esi,%edi,4)` at 6dad7f, `eax = [game+0x550]`.
- `Leader::init (006e3930)`: `movl %eax, 0x1dc(%esi)` at 6e3d57 (`0xd4 + 0x1dc = 0x2b0`), 0, j in 0..8.
- Block writers: `Leaders::clear`, `LeaderData::walk_data`.
- False positives checked: `LeaderOut::warn_resources` 6d36a3 (`andl $0, 0x2cc(%eax)` with `eax = [0xc06210]`, the console object, not a leader), `Unit::fight` 5ff29d/5ff2b8 (a `Unit`), `Leaders::clear` 6ed809–6ed827 (BitMask clears), `Options::*`, `Game::restore_type_backup` (other records).

**2. Event, step, predicate.** `broke_alliance[target] = frame` (a sim frame number, `game->frame`), on the declarer only, whenever `action_declare` gets past its refusals while the two are still allied both ways — i.e. every break of an alliance, by anyone: the computer's level-1 declarations in `Leader::diplomacy` (6beceb, 6bed1c), a computer's level-0 war on an ally (turned into level 1), the refuse-2 arm of `action_respond` (6d04ed), a human's interface declare, `DropControl` (war on an ally with force). **No `quiet` gate** (`quiet != 0` jumps from 6dacf0 straight to 6dad68, the stamp's test). The target's mirror slot is not written.

**3. Reset.** `Leader::init` → 0; `Leaders::clear`. Otherwise only overwritten by a later break.

**4. Readers.** `Leader::action_declare` itself (`max` of both directions against `ally_to_war_delay`, step 2 above); `Unit::process_attrition (005e11a0)` (decompile lines 164–167: both directions against `ally_to_war_delay`, and `ally_to_war_grace` for the grace window). `docs/` already states (`crates/sim/src/territory/attrition.rs:431–437`) that both constants are absent from `rules.xml` and load as -1; with a delay of -1, `frame - t < -1` is never true, so with shipped data both readers are dead and the field is behaviourally inert, but it is written and it is in the checksum list (`crates/sim/src/checksum/leaders.rs:81`).

**5. Crate.** Not carried. The relation change it rides on is `Sim::set_diplo` level 1 from allied (`crates/sim/src/leaders/relations.rs:116`, the `_` arm) — but the stamp belongs to `action_declare`, which the crate does not have; `attrition.rs`'s `in_war_grace` input is where a reader would consume it.

**6. Plain game?** Yes when an alliance breaks: a computer leader breaking with an ally in `Leader::diplomacy` (needs a game where alliances form and can break — diplomacy team styles), or a human breaking one. Under locked teams (`TEAM_STYLE 1`, the captures' style per `diplomacy.rs:10`) I did not establish whether the level-1 arms are reachable; not read.

**7. Confidence.** High on the write and its predicate. Not read: which `Leader::diplomacy` predicates lead to 6beceb/6bed1c (the decompile shows 6beceb behind `who == target`, which reads oddly; I did not resolve it).

## `made_peace` (+0x2d0, int[8])

**1. Writers.**
- `Leader::action_respond (006d03c0)`: `movl %eax, 0xe3a660(,%ecx,4)` at 6d108d (`ecx = responder*0x1bbb + proposer`) and at 6d10a0 (`ecx = proposer*0x1bbb + responder`, `-0x20(%ebp) = proposer*0x1bbb`), `eax = [game+0x550]` each time.
- `Leader::init (006e3930)`: `movl %eax, 0x1fc(%esi)` at 6e3d63 (`0xd4 + 0x1fc = 0x2d0`), 0.
- Block writers: `Leaders::clear`, `LeaderData::walk_data`.
- False positives checked: `LeaderOut::warn_resources` 6d36ad (console object), `MapEastMeetsWest`/`MapWarringStates::make_continents`, `IFaceDiploBase::setup_chat`, `ScenarioFuncSet::set_unit_type_max_craft`, `Game::restore_type_backup` (other records).

**2. Event, step, predicate.** Both sides' slots are stamped with the sim frame: `responder.made_peace[proposer] = proposer.made_peace[responder] = frame`. `action_respond(this = responder, proposer, response)`. Only `response == 1` (accept) reaches it, through the apply arm (`LAB_006d0df9`; for a proposed alliance only when the team-size test passes). The proposal's treaty word is `responder.dip[proposer].treaty` (`LeaderData + 0x692c + proposer*0x5c + 8`, `Diplomacy::treaty`). If it is not -1, both directions are `set_diplo`'d to `max(treaty, current diplos)` (an accepted deal never lowers a relation), and **if the treaty word is exactly 1** the two stamps are written — even when the `max` leaves the pair allied, so a level-1 clause accepted between allies still stamps.

**3. Reset.** `Leader::init` → 0; `Leaders::clear`. Overwritten by the next accepted peace.

**4. Readers.** `Unit::process_attrition (005e11a0)` (decompile lines 171–173, listing 5e1697/5e16d0): within `ally_to_war_delay` of either side's stamp, the peace grace. With the shipped -1 (see `broke_alliance`) the test never fires.

**5. Crate.** No proposal/accept path: `crates/sim/src/leaders/diplomacy.rs:177` (`SEAM: dip[t].agree … the Diplomacy records are not carried`). A builder adds the stamp to a new `action_respond` beside its `set_diplo` calls (`Sim::set_diplo`, `crates/sim/src/leaders/relations.rs:116`).

**6. Plain game?** Yes where peace treaties are offered and accepted: callers of `action_respond(.., 1)` are `Leader::diplomacy` (6be848, 6bf3e5, 6c0888, 6c08ea, 6c0d81, 6c0f40, 6c1299), `Leader::process_taunt` (6b8f16), `Leader::action_agree`, `CommandPackage::process_accept` (the human), and `ScenarioFuncSet::accept_offer` (triggers). It needs a lobby where leaders start or fall out of war and negotiate (diplomacy team styles); whether the locked-team captures ever reach an accepted treaty-1 proposal I did not establish.

**7. Confidence.** High on the stamp and the treaty-1 predicate. Not read: which proposals carry `treaty == 1` (the offer builders `Leader::action_treaty`, `action_offer`), and the team-size arm for `treaty == 2` beyond its branch target.

## `hire_stamp` (+0x1b4, int[8]) and `hire_who` (+0x1d4, int[8])

One writer steps both, so one section.

**1. Writers.**
- `Leader::action_respond (006d03c0)`, the hire loop (listing 6d11bc–6d12a9):
  - `movl %eax, 0xe3a544(,%ecx,4)` at 6d121e — `responder.hire_stamp[X] = frame`;
  - `movl %eax, -0x20(%ecx)` at 6d122e, `ecx = [0x8(%ebp)] = &proposer.hire_who[X]` (6d11d4 `leal 0xe3a564(%eax)` with `eax = -0x14(%ebp) = proposer*0x6eec`, set at 6d0903; stepped by 4 per X at 6d12a0) — `proposer.hire_stamp[X] = frame`;
  - `movl %edi, 0xe3a564(,%eax,4)` at 6d123a — `responder.hire_who[X] = proposer` (`edi` = proposer);
  - `movl %eax, (%ecx)` at 6d1244 — `proposer.hire_who[X] = responder`.
- `Leader::init (006e3930)`: `movl %eax, 0xe0(%esi)` at 6e3c2d → `hire_stamp[j] = 0`; `orl $-1, %eax; movl %eax, 0x100(%esi)` at 6e3c33/6e3c36 → `hire_who[j] = -1`.
- Block writers: `Leaders::clear` (both to 0, so `hire_who` reads 0 until `Leader::init` runs), `LeaderData::walk_data`.
- False positives checked: every `ScenarioEditor::*`, `IFaceEditorNations::*`, `IFaceDiploBase::*`, `ConquestGame::*`, `ConquestLeader::demand_tribute`, `Objects::init`, `ScenarioRead::load_leaders_chunk`, `ScenarioFuncSet::set_object_type_attack` hits are on interface, editor, conquest, object or game-info bases; `Leader::plan_strategy`'s `leal 0x1e8(,%ecx,4)` (6b9ab5) is an index scale, not the field. The export's `pLVar16->hire_who[4] & 0x4000000` in `BuildOut::make_options` is a read through a mistyped pointer (offset `0x1e4`), not a writer.

**2. Event, step, predicate.** On an accepted proposal (`response == 1`, the apply arm as for `made_peace`), after the treaty and goods are applied, **if `Game::war_allowed (00594670)`**: for each slot X in 0..8 with X ≠ responder, X ≠ proposer, and `responder.dip[proposer].attacks[X] != 0` (`Diplomacy::attacks`, `+0x3c`; listing `0xe40cf8(,%eax,4)` with `eax = responder*0x1bbb + proposer*0x17 + X`):
- both sides' `hire_stamp[X] = frame` (sim frame number);
- `responder.hire_who[X] = proposer`, `proposer.hire_who[X] = responder` (a slot index);
- then each of the two, if not already its enemy, `action_declare`s war on X (level 0, quiet = `!(HUMAN)`, force 0 — read at 6d1264–6d1273 and 6d1288–6d1297).
The stamps are written before the declare and whether or not it succeeds. No per-X exemption beyond the three listed.

**3. Reset.** `Leader::init` (`hire_stamp` 0, `hire_who` -1); `Leaders::clear` (0, 0). Overwritten by the next hire against the same X.

**4. Readers.** `Leader::diplomacy (006bc950)`: `hire_stamp[target]` non-zero and within 13500 frames (`0x34bc`) forces `aggression = 1` (decompile lines 925–928; listing 6bdc8a); within 4500 frames (`0x1194`) it vetoes a coin that would clear aggression (lines 933–934; 6bdce2); a third leader's `hire_who[target] == candidate` with its `hire_stamp` within 9000 frames forces aggression (lines 949–950; 6bdd57/6bdd6b); `hire_who` is also read through `puVar29` off `&field_0x1d4` (line 943) and `-0x1dc` (line 1845; 6bf7a5) in the ally-help arm; and 13501 frames (`0x34bd`) at line 1346 (6be680). Frames are sim frames.

**5. Crate.** Not carried: `crates/sim/src/leaders/diplomacy.rs:453–454` (`SEAM: hire_stamp/hire_who … hiring is not modelled`). No `action_respond`; a builder writes both in a new `action_respond`'s apply arm.

**6. Plain game?** Reachable in a skirmish whenever an accepted proposal carries an attack clause and war is allowed: computer–computer and human–computer deals through `Leader::diplomacy`'s accepts and `CommandPackage::process_accept`. It needs a diplomacy lobby with three or more leaders where proposals are exchanged; not reachable in a two-player game (X must be a third slot). Whether any capture reached it I did not check.

**7. Confidence.** High on the four stores, their bases and the predicate (all from the listing). Not read: who builds a proposal with `attacks[]` set (the offer builders), and the team-size arm.

