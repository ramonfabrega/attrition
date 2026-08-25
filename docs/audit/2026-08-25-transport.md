# Audit: transports and docks — the second reading (2026-08-25)

**Document:** `docs/TRANSPORT.md` (first reading in the main thread on
Fable, the same day). **Blind readers:** two, on Opus 5 (verified from the
transcripts, `claude-opus-5`, 76 and 71 turns), one wave, run in parallel
with run22 — A on the leader's level, the unit bits and
`think_civilian_transport` (45 claims, `A.0`–`A.44`); B on the docks
registry, `is_dock_tile`, the dump writers, the army's transporting step
and the coast masks (69 claims, `B.1`–`B.68` with `B.55a`). Reports at
`~/ghidra-projects/reading/transport-2026-08-25/`, brief `BRIEF.md` there.
**Adjudication:** the main thread on Fable, against the decompile and the
listing, the same hour. **Capture:** run22 (`docs/ORACLE.md`) landed
between the brief and the verdicts and settled two rows neither reading
could.

**Disclosure.** Both readers were handed `CLAUDE.md` and the memory index
by the harness, as every subagent is; both report that neither names a
transport fact, and A adds the `gitStatus` block's five commit subjects
(`sea_map`, the islands map — nothing of this mechanic). No leak-affected
row.

## Headline

The arithmetic and the predicates were doubly confirmed almost everywhere,
which is the pattern of every audit so far. Six verdicts changed the
document, two of them changed Rust, and the two most consequential facts of
the day came from **neither reading but the capture**: a dock registers in
the *sea* region, so `reg_docks` is never counted for it (B.9 called it
"the cheap 'do I have a dock in region R' query"; the first reading said a
dock's cell "always is land"; run22 says `reg 65`), and owner 9's gull is
not in a `DUMP_ALL` block at all. The lesson is the one `CLAUDE.md` already
states — prefer a diff to a reading — with a sharper edge: **a claim about
which side of a shore a building's centre falls on is a map fact, and no
amount of code reading settles a map fact.**

Where the readings did differ from the document, it was the familiar
places: a *scope* error (the `domain != 2` test wraps the gate, not the
search — A.23), a *name* error (`+0x14` is `num_captains`, not `num_units`
— B.51/B.61), a *writer* omission (`gull_o` is not cleared — B.13), and a
*dead clause* nobody had noticed (`num_coasts` on a sea region is 1, so the
census's "more than one coast" test never holds — B.41).

## Verdicts — Reader A

Verdict letters: **A** the document, **B** the reader, **both** doubly
confirmed, **neither**, **open**. A verdict is a function and an expression.

| row | claim | verdict | settled by |
| --- | --- | --- | --- |
| A.0 | `0x5222 + 2t` is `num_buildings[t − 414]`, `0x11a2 + 2t + 0x102r` is `reg_buildings[r][t − 414]` | both | `types.txt` `+0x555e`/`+0x14de`; `BASE_BUILDTYPES 414` |
| A.1 | the lock is `leader_flags2 & 0x20`, read from `leaders.list[who]` | both | `locked_transport@006d5230` |
| A.2 | SCOUT 1, MILITARY 2, CIVILIAN 3; highest bit wins | both | `can_transport@006e0c60` |
| A.3 | `check_transport` only ever sets 3 or 0 | both | `neg/sbb/and 3` at `006bc611` |
| A.5 | the second `has_preq` is redundant | both | control flow, `006bc690` |
| A.6 | dock count runs the `to` chain; DOCK→ANCHORAGE→SHIPYARD | both (chain: data) | `get_buildings@006e0680`; `buildingrules.xml` `FROM` |
| A.7 | level-unchanged early-out; the unit loop runs only on a change | both | `006bc688` |
| A.8 | the clear loop has no filter but `flags & 1` | both | `006bc7c7`ff |
| A.9 | the nine callers; `set_transport` modes 1/2 skip `check_transport` | both (+addition) | the export |
| A.10 | the editor's three states | addition | `ScenarioEditor::set_transport@0099bc20` — scenario-only, recorded |
| A.12 | vslot `+0x18` is the folded `return 1`, so `can_ever_transport` is land ∨ (carry ∧ ¬carrier) | B | `Buffer::is_pending_load@0041e0e0` is `return 1`; the document's "vslot, the folded constant" said the same less plainly — §3.3 reworded |
| A.13 | domain 0/1/2 | both | `is_plane@0046ce40` |
| A.14 | `transport_type`'s set; merchants by exact index, scouts/caravans by lineage; `0x190` is `FURTRAPPER` | B on the name | `enums/TypeIndex.txt:426`; §3.1 named the three |
| A.15 | `can_transport`'s three terms | both | `0046f960` |
| A.16 | `unit_masks2 & 0x2000` written only by the two `ScenarioFuncSet` functions | both; A's "other writers" open | the export — no other `\| 0x2000` on `+0x6c` found by either reading |
| A.17 | the four writers of the unit bit | both | grep of `0x800000` |
| A.18 | `Unit::init` and `check_transport` share the three conjuncts; option bit 1 is the scout exemption | both; the option's name medium in both | `Unit::init@00612100:318–338` |
| A.19 | `action_set_transport` tests neither `transport_type` nor the scout option; skipped for building groups (A.42) | both | `007024b0`; `GroupData +0x49 buildings` |
| A.20 | `force_transport_ability` sets lock + level and every unit's bit | both | `009ffb00` |
| A.21 | `cast_transport`'s type choice and refusal; `find_nearby_spot == 0` is the found case | both; the sense settled | the `else` arm plays `S_NOT_NEAR_OCEAN_FOR_TRANSPORT`, so non-zero is failure |
| A.22 | the two callers and `param_1`'s meaning; `0x40000` is set at `Unit::init` when the owner is not `leader_flags & 0xc == 4` (A.43) | both | `think_peasant:53`, `think_scout:580`; `docs/ORDERS.md` names `0x40000` AI-controlled |
| **A.23** | **an air unit skips the capability gate and runs the search** | **B** | `if (domain != 1 \|\| flag) { if (domain != 2) { gate } search }` — the document had "domain 2 always [refused]"; **§7 corrected** |
| A.24 | gate 2 = level ≥ kind ∧ `can_transport` ∧ `is_cargo` | both | `005f40d0` |
| A.25 | gate 3's three conjuncts; the quota `xport_peasants < city_num` | both | `+0x9c0 < +0x3f8`, `WData.who`, `reg_cities` |
| A.26 | land 1..63, sea 65..126 | both | loop bounds `0x2200`, `0x2288..0x4378` |
| A.27 | candidates by errand; `go_here` called twice on one path | both | `005f4327`ff |
| A.28 | a shared sea region, `is_coast` both ways | both | `0x7f` sentinel |
| A.29 | `is_coast` reads the land side's `coast` in both directions | both | `00680f90`, `+0x64` |
| A.30 | no draw in this half | both | grep; the callees open in both |
| A.31 | the stride and the `(o + frame) % step` phase | both | `005f4421`; §7 |
| A.32 | the accepted cell: `coast_here` ∧ `num_waterhalf == 0`, i.e. the land-side shore cell | B (meaning) | `num_waterhalf@006b4db0`: 0 unless `flags & 0x100`, else the water-tile count — §7 now says so |
| A.33 | `coast_here` returns the direction 1..8 and swaps the sought region | both | `00681020` |
| A.34 | score `vector_dist(\|dx\|,\|dy\|) × max(1, danger)`, minimised, negatives dropped | both | listing `005f4578` |
| A.35 | one-unit group, `action_move_near` to the cell centre, `EXPLORE_TO`/`MOVE_TO` | both | `005f40d0` tail |
| **A.36** | **the booking indexes the destination cell's own region, which `coast_here` lets be the sea's** | **B (addition)** | `wdata + 4` at the chosen cell; §7 and §13 now carry it — a sea index writes past `reg_xport_peasants[64]` into `reg_cities`; whether `num_waterhalf == 0` admits one is open |
| A.37 | the fall-through | both | — |
| A.38 | `go_here` bit 1 | both | `006810f0` |
| A.39 | `Region::flags & 8` from `analyze_map`; the threshold | both | `docs/AI.md` §15.8 has the same rule from the same function |
| A.40 | `go_here` bits 2/4 | both | `0xe3b5ee`/`0xe3adf8` = `reg_cities`/`strategy` |
| A.41 | `needs_transport` returns `(¬w1) + 1` | both on the expression; **A.41's prose inverted** | `(uVar1 != 0x20) + 1`: first tile water → 1 (disembark), land → 2 (embark); the reader's own formula says so and its sentence says the reverse |
| A.44 | `unit_masks & 0x100` — auto-explore, writer unfound | open, both | not this mechanic's |

## Verdicts — Reader B

| row | claim | verdict | settled by |
| --- | --- | --- | --- |
| B.1–B.4 | 20 slots per leader; `dock_mark` is the live count; the lowest free slot below the mark is reused; `mark = max(mark, slot+1)`; the slot on `BuildData +0x78` | both | `00741580`, `00740fc0` |
| B.5–B.6 | created only from `Build::activate` (not-fort ∧ dock), closed only from `Build::close`; `is_dock` is `is(0x1b0)` | both | `00623e20:558`, `00628980:225` |
| B.7 | `dock_flags = 1` outright | both | `00740a80` |
| B.8 | `reg` is the building's cell region | both | `>> 8` through `div_3_table` |
| **B.9** | `reg_docks` "is the cheap 'do I have a dock in region R' query" | **neither — run22** | the dock's `reg` is **65**, the sea; the `< 0x40` guard skips it. The arithmetic is right in both readings; the interpretation was wrong in both. `docs/TRANSPORT.md` §5.2, §13 |
| B.10 | the gull: who 9, `GULLBIRD`, one tile up-left | both | `0x194`, `−0xc0` |
| B.11 | exactly one `Random::get` in `Dock::init` | both, and **two draws per dock** in total | run21/run22 frame 3579: `Guy::init_real` (the gull's creation, inside `init_unit`) then `Dock::init+0x125`; B scoped its count to `Dock::init`'s own body |
| B.12 | the strafe order addresses the dock object | both | — |
| **B.13** | **`Dock::close` does not clear `gull_o`** | **B** | `007409f0` writes `o`, `reg`, `who`, the bit — not `+0x6`. `transport.rs::dock_close` cleared it; **Rust changed**, test extended |
| B.14 | `close_dock` trims the mark from the top | both | `00740f50` |
| B.15 | Ghidra's `build_list` in `remask_docks` is `docks` | both | `rise_z.map` |
| B.16–B.19 | the water apron: box `[−3, size+3)`, `0x2000` + `WData.bad`; the same bit `set_blocked_at` keeps; odd-footprint nudge; `remask_docks` restores overlaps after `mask_me`'s unmask | both (+B settles §13's "what clears the apron") | `00740c90`, `006312a0`, `006b4900` |
| B.20–B.21 | `find_dock`: every leader, nearest, `0x200` = same region | both | `0065cfd0` |
| B.22 | `find_dock`'s `vector_dist` args | open | listing not read by either; no caller in the game |
| B.23–B.28 | `is_dock_tile`: cells; third arg unused; water cell not `0x100`; N/E/S/W from `orthog[1..4]`; the asymmetric one-tile step; the shore tile not mountain/forest; the 4×2 strip | both | `00636700`; the tables from `.rdata` in both readings |
| **B.29** | `is_dock_tile`'s only caller is `plan_strategy` — AI-only | **B (addition)** | grep; §5.6 and §13 amended: no placement caller, run20's `dock_tile` is its whole check |
| B.30–B.31 | the `DOCK` record's six fields in order; 8 × 20 records | both | run20/run22 (`length 20` per leader; 160 records) |
| **B.32** | sea regions start at `0x41`; land at 1; 0 and `0x7f` excluded | **B (addition)** | `Regions::find_all@0067eff0`, `clear_all@00680060`; §9 amended — the `0x40` skip in `num_coasts` is exact, not an artefact |
| B.33–B.34 | water is `land ∈ {1,2}`; `reg_combat[127]` is the one sea-indexed leader array | both | `docs/AI.md` §2.3 step 8 |
| B.35–B.38 | `coast` = the seas a land region touches (ring 1); `coastal` = land regions within rings 2–3 across water, closed by `finalize_coastals` through a shared sea | both | `0067fd70`, `006805b0` |
| B.39 | `coast` bit 63 never consulted | addition, harmless | — |
| B.40 | `is_coast` | both | — |
| **B.41** | **`num_coasts` on a sea region is 1**; the census's `> 1` clause is dead | **B** | the `region == i` arm; `World::num_coasts` now answers 1 for a sea region — **Rust changed** |
| B.42 | `set_coastals` from `find_all` only | both | — |
| B.43 | `div_3_table[−2]` on a −1 neighbour | open | harmless unless the bytes before the table say otherwise |
| B.44–B.47 | `status` bits and dispatch order; the 128/256-frame cadence; `human_frame`; the defeated bit `0x40` | both / addition | `006f93d0` — the cadence is `docs/ARMY.md`'s to own |
| B.48 | `leader_flags & 1` in use, `& 4` human | both | `docs/ORDERS.md` §8 |
| B.49–B.50 | the gate is `& 0x300`; `0x40` is written only in `do_mustering` | both | — |
| **B.51** | `do_mustering`'s table; **`num_captains > 7`** | **B on the field** | `+0x14` is `num_captains` (`types.txt`); the document had `num_units` (`+0x10`); **§8.1 corrected** |
| B.52 | the common tail aims at the muster cell's centre | both | — |
| B.53–B.54 | `do_transporting` moves nothing; runs only when the army is in its region | both | — |
| B.55, B.55a | the scoring loop; **`dist` is between the two regions' first cells, in cells** | both | listing `006f476f`–`006f478e`, read by both sides independently |
| B.56 | `city_mark` bounds the scan | B (addition) | `+0x408`; §8.2 amended |
| B.57–B.58 | no city → `find_target`; else own city, muster spot, the y-nudge | both | — |
| B.59 | `init_army` evicts the smallest | addition | `docs/ARMY.md`'s |
| B.60 | `init_navy` from `create_units` only, region = the sea index | both | — |
| **B.61** | `send_navy`: `reg > 0x3f`, `navy`, **`num_captains > 2`** | **B on the field** | as B.51; **§8.3 corrected** |
| B.62–B.65 | the two-sided coast test; `process(0)` keeps the cadence; the region-id parameters; land armies only | both | — |
| B.66 | `go_here`'s bits as `do_transporting` weighs them | both | — |
| B.67 | the merge path excludes navies | addition | `docs/ARMY.md`'s |
| B.68 | the `ARMY` record's fields | both | run20 block 1 |

## What changed

**In `crates/sim`:** `transport.rs::dock_close` keeps the slot's `gull`
(B.13), asserted by `a_dock_with_a_gull_type_draws_twice`;
`world.rs::num_coasts` answers 1 for a sea region (B.41), asserted by
`coasts_are_computed_from_the_cells`. Neither moves a run: the census's
`big` test never depended on the dead clause.

**In `docs/TRANSPORT.md`:** §3.1 (the merchant names), §3.3 (the folded
slot), §5.2 (the sea-region finding, from run22), §5.3 (`gull_o`), §5.4
(the apron's eraser), §5.6 (the only caller), §7 (the air unit; the
land-side cell; the sea-index booking), §8.1 and §8.3 (`num_captains`),
§8.2 (`city_mark`), §9 (`0x41`, `num_coasts`), §13 (two items struck, one
added), §14.

**Still open, named:** `find_dock`'s distance arguments (B.22, no game
caller); the sea-index booking's reachability (A.36); `unit_masks & 0x100`'s
writer (A.44); other writers of `unit_masks2 & 0x2000` (A.16); the
`div_3_table[−2]` read (B.43); whether any shipped dock ever has a land
centre cell (run22's finding, `docs/TRANSPORT.md` §13). The cadence, merge
and eviction rows (B.45, B.59, B.67) are banked for `docs/ARMY.md`.

## Method note

The blind reading was cheap — two readers, one wave, ~23 minutes each in
parallel with a behavioural run — and it did what the second readings have
done before: names and scope, not arithmetic. What it could not do is what
the capture did in its first `grep`: tell which region a real dock's cell
is in. Both readers wrote "the region of the dock building's own cell" and
both let the reader assume land, because that is what the guard's shape
suggests. The order that worked today — first reading, capture staged
from the trace's own frame numbers, blind readers briefed while it runs,
adjudication against both — is worth repeating as written.
