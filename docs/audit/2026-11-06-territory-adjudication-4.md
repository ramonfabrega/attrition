# Territory adjudication 4 — compared-field claims, unit/wall/world stems

The third reading of `adj-4.tsv` (14 rows, all `CLAIM`: both readers agree the crate differs, and
at least one says the field that would show it is printed and compared). Adjudicator: Opus 5.5,
2026-11-06. First readings: `docs/audit/2026-11-06-territory-unit-1.md`, `-borders-2.md`,
`-world-3.md`; second readings: `-second-5.md`, `-second-6.md`. Bodies read in
`~/ghidra-projects/decomp/funcs/`; the instrument in `crates/rondata/src/diff/`.

**What "compared" means here, read once for every row.** `coverage.rs`'s two pins are per *key*,
not per *bit* or per *state*: a key leaves `UNREAD` (`coverage.rs:85`) when any parser reads it,
and leaves `UNCOMPARED_BY_THE_INSTRUMENT` (`coverage.rs:3381`) when any site calls
`compared::note` on it in any driven window. So a key off both pins can still be blind to the
case at hand, three ways, and each row below says which applies:

- **masked**: `unit_masks` is compared for bits `0x2`, `0x10`, `0x1000`, `0x10000`, `0x80000`,
  `0x100000`, `0x2000000` only (`harness.rs:85`, `:286`, `widen_block_rows` `harness.rs:2111–2185`);
  `unit_masks2` for `0x10 | 0x200 | 0x800 | 0x20000` (`harness.rs:276–284`) and `0x40000`
  (`sheltered`, `harness.rs:2088`). `unit_masks & 0x800` and `unit_masks2 & 0x8000/0x40/0x20` are
  printed, parsed (`gamelog.rs:3506–3507`) and compared nowhere.
- **gated**: `BuildDump job_counter` is compared only while both sides call the site unfinished
  (`harness.rs:687`, `.filter(|_| !ours.active && b.flags & 4 == 0)`).
- **widening-only**: `myspeed` is a row of `widen_block_rows` (`harness.rs:2078`), run on a word's
  window, not of the per-frame `compare` (`harness.rs:11`); it scores no floor by itself.

And what parts *lockstep*: `Report::ticks_before_divergence` (`report.rs:299`) counts position
rows only, `order_ticks_before_divergence` order and path rows (`report.rs:330`). A hit-point,
mask or speed row parts the per-frame record (`hits_diverged`, `collide_diverged`) without moving
either score until it moves a position, an order, or kills a unit. A `CLAIM` row is **confirmed**
when its field is compared on the per-frame record (or by an order/position consequence on the
case's first frame) and the case is unreached.

**And what a "lockstep game" is.** `tools/rubric.py:14` — a game whose pinned *draw-stream* word
stands at its full length (`battery::Lobby`'s `count`/`sequence`, `third::walk_from`
`third.rs:124`, which ticks the sim against the trace's draws). The record compare runs only on the
capture's dumped blocks — a `cover=1` twin dumps a closing window (run1017: 15230..15433 of 15432)
or a stride (run1492b: every 100th frame) — and on a widened window the record rows that part are
**pinned as a count**, not asserted zero (`airbase_melee.rs:309`, `w.firsts.len()` 181 on run1492's
word window, a game whose draws agree whole). So "entered by a lockstep game" (the TSV column) is
*function* coverage, says nothing about the arm, and a compared record field can already disagree
on a lockstep game without anything going red until it moves a draw.

## Part 1

| va | name | kind | first | second | verdict | settles it | the item |
|---|---|---|---|---|---|---|---|
| 005e6b80 | Unit::do_garrison | CLAIM | twin-differs (helicopter arm) | differs (helicopter arm; no `nocoll` retry, no dock ring, redirect on `NoCapacity`) | **confirmed**; B's three extra deltas also stand | `do_garrison@005e6b80:63–83`: `(type+0x2b4 & 0x20) != 0` and the target `is(..)` → `can_carry` → `kill_current_order; add_strafe_order(..,1,QUEUE_FIRST,1); +0x3c = 1`, else a `QUEUE_FIRST` move + red feedback. The crate (`crates/sim/src/orders.rs:8165–8211`) goes straight to `garrison()`. Extra: `:180–205` calls `find_nearby_spot` a second time with arg `1` (nocoll) after the first fails, `:166–170` widens the ring to `+0x1b0..+0x330` for a building that `is(..)`; `:124` a zero `get_garrison_limit` jumps to `LAB_005e70c0` (`kill_current_order`, `:363`) where the crate's `GarrisonRefused::NoCapacity` (`garrison.rs:251`) takes the `search` redirect (`orders.rs:8198`); `:281` gates the redirect on the city's `+4 & 1`. (a) The order list kind (`STRAFEORDER` vs `GARRISONORDER`) is compared per frame and scores `order_ticks_before_divergence` (`report.rs:330`); `inside_up` is compared (`inside_diverged`). (b) No lockstep game entered the function at all (TSV `-`), so no arm of it was reached; the crate's garrisons on chapter captures enter through other paths. | ~~a capture of a Helicopter ordered into its own Airbase (an Industrial+ lobby, `@garrison`): order score parts on the order's first frame~~ answered by item 3990 (`docs/GOLDEN.md` §183, run1548): `0/9` holds a `STRAFEORDER` home to `0/2007` on 1262 and is inside from 1318, built in `do_garrison_order`; the three extra deltas stay unbuilt |
| 005e8de0 | Unit::pitch_aircraft | CLAIM | twin-differs (flying-target arm) | differs (flying target, helicopter, carrier-home `local_2c`) | **confirmed**, B's wider list stands | `pitch_aircraft@005e8de0`: the flier arm `*(int*)(target type + 0x218) == 2` → the target figure's `+0x14` z `± min(300, max(0,(0x600 − dist)·5))` by object order; the helicopter `local_20 = type+0x2b4 & 0x20` steps `z` by 25 (`local_14 = iVar2 * 0x19`) and leaves pitch 0; `local_2c = (*param_4 < carrier_speed·3)` blocks the inside-`0x600` halving. Crate: `air.rs:2197` SEAM (flier), `air.rs:865` `if self.is_helicopter(u) { return Flew::Done }` (a helicopter is never flown), carrier home absent (`air.rs:856–860`). (a) Figure `z`/`last_z` are off `UNREAD`'s `UNITDATA/GUY` row and off the `Guy` uncompared pin (`coverage.rs:3419–3423`): compared on the air windows (`docs/ORDERS.md` §33.7); the position consequence is compared per frame. (b) No compared window holds a strike on a flier, a helicopter in an air order, or a plane homed on a carrier (`docs/ORDERS.md` §42.3 row 1, parked 3446a); run1492's 27 stride blocks hold no `STRAFEORDER`/`AIRPATROLORDER`/`AIRATTACKGROUNDORDER` at all (grep), so the airbase melee, whose Carrier `1/18` sank on 1390 (GOLDEN §156.1), shows no plane in flight on a dumped frame. | a capture with a Fighter striking an enemy plane (`U3`, the figure's `z` on the strike's frames); the helicopter and carrier arms ride the do_garrison and has_repeat_air captures |
| 005e9be0 | Unit::check_fuel | CLAIM | twin-differs (re-home search, out-of-fuel death) | differs (same; plus carrier home and helicopter) | **confirmed** | `check_fuel@005e9be0:60–136`: home gone or `!can_carry` → nearest `can_carry` building, then unit, by `vector_dist`, rewritten into the order; none → `(type+0x2b4 & 0x20) == 0` → vslot `+0x158 (2,-1,0)` (`Unit::die`), `return 1`; a helicopter aims at its own point. Crate: `air.rs:884–887` `air.home.filter(|&b| self.buildings[b].alive)` else `return Flew::Done` — the plane is left where it is, no search, no death. (a) The air order's `oxx`/`whose` are read (only `STRAFEORDER/AIRORDER/UNITORDER flags` is on `UNREAD`, `coverage.rs`) and compared by the order row; the death by `death_diverged`, the position per frame. (b) The function is entered by six lockstep games but the arm needs a returning plane whose home died or filled; no document records one (the SEAM `air.rs:856–860` is "never reached"), and run1492's stride holds no air order (row above). Whether one of run1019–1022/1176/1177 razed an Airport with a plane out is not established — their `cover=1` dumps carry only closing windows. | ~~a capture razing an Airport while its plane is out (`U3`, `DEATHS`): the plane's position and its death on the empty-tank frame~~ answered by item 3990 (`docs/GOLDEN.md` §183, run1548): the three planes re-home to `0/2008` by 1216 and die on 1231 once it is razed, built as `rehome` and `out_of_fuel`; the carrier loop parked 3990a |
| 005f2f30 | Unit::think_carry_idle | CLAIM | unmodelled | missing | **confirmed** — the one row whose arm a lockstep game **did** enter | `think_carry_idle@005f2f30:58–79`: `find_nearby_spot` at the nearest live coastal `DOCK` city, and unless the spot is the boat's own cell on both axes (`div_3_table[..>>8] != ..` at `:65–67`) a `MOVE_TO` to `cell·0x30 + 0x18`, `return 1`. Crate: `transport.rs:1921–1924` `fn idle(&mut self) -> i32 { 0 }`. (a) The boat's `MOVEORDER` and `orders_x/y` are compared per frame. (b) Entered by run1483b and run1492b, whose draws agree whole; the two possible reasons are both unmeasured: the spot test refused (the boat already beside its dock city: `return 0`, nothing made), or the boat's walk drew nothing — a walking boat spends no draw until it fights — and its order row sits inside the stride windows' pinned `firsts` counts (181 on run1492's word window). Which, is the item. | a test run on disk: `RON_FIRSTS=1` on `airbase_melee_run1492_s_word_window_is_widened_whole` and the sea melee's twin — a boat's `orders`/`orders_x` row in the list names the case; then wire `carry_idle.rs` (parked 3411a) |
| 005f3260 | Unit::think_carry_military | CLAIM | twin-differs (no-army and no-target arms) | differs (same) | **confirmed** | `think_carry_military@005f3260:29–42` (no army: `eject_contents(0,-1,0,1)`, an empty boat `die`s) and `:118–121` (dead or no target: `regions[r] + 0x7c`, or the city of a region its leader marks `& 4`). Crate: `transport.rs:640` doc names both as SEAMs answering `false`; `think_carry_army` carries the live-target and ashore arms. (a) Boat `MOVEORDER`, riders' `inside_up`, deaths: compared per frame. (b) Entered only by run1179 (a long Russian late-age game, `cover=1`, closing records); the two arms need a soldier boat whose group lost its army or whose army's target died while the boat sailed — not on any compared window (the documented case, run1066 on 770, is a staging take with a live target). | a capture of a soldier barge whose army's target dies in transit (`U3`, the boat's order on the next frame) |
| 005f5650 | Unit::think_caravan | CLAIM | twin-differs (`param_1` skips the gate) | differs (same; the threshold reads the unit's own `0x40000`; `do_trade`'s other kill arms re-think in the original) | **confirmed**, both of B's extras stand | `think_caravan@005f5650`: `if (param_1 == 0) { threshold = (unit_masks & 0x40000) ? 1 : option table; idle gate }` — the gate is skipped on `do_trade@005ed270:649` `think_caravan(this_03, 1)`, reached by the shared tail `LAB_005eda11` (`:646`, ten `goto`s). Crate: `caravan.rs:145–170` takes no argument and gates on `self.ai_driven(owner)`; `caravan.rs:425` re-thinks only on the dead-city arm, and `:296`, `:308`, `:323` kill without re-thinking. (a) The order list (a `TRADEORDER` present or not, its `TARGETORDER` `ox`/`whom`) is compared per frame; the trade order's own `oxx`/`whose`/`loaded`/`started` are on `UNREAD` (`coverage.rs`, `UNITDATA/TRADEORDER`). (b) For an AI caravan the gate costs one frame (threshold 1: `idle` reaches 1 on the next frame); for a human ~12. A caravan's order spends no draw, so a one-frame lag on a lockstep game parts nothing but a record row on a dumped frame, and no compared window holds a route ending on a city's death. | a dump grep first: a lockstep closing window (run1017 15230..15433) with a city captured and a caravan of it — the frame its `TRADEORDER` goes; else a capture of a razed trade city |
| 006055c0 | Unit::update_speed | CLAIM | twin-differs (Aztec arm) | differs (Aztec arm; constant's load unknown) | **refuted as a live difference — inert on stock data**: the crate does not carry the arm and the stock game never exercises it | `update_speed@006055c0:155–160`: types `0x32`/`0x33` with `has_tribe_bonus(0)` → `v = (aztec_move_speed + 100)·v / 100`, last. `AZTEC_MOVE_SPEED` is absent from `Data/rules.xml` (grep). **Measured**: run1256b (East Indies, who=1 the Aztecs, tribe 0) frame 8711 — all 51 who=1 gatherers print `myspeed 25`, the same as who=0's nine Nubians; with the constant at −1 it would be `99·25/100 = 24`. So the constant loads in 0..3, and at the citizen's only stock speed (25) every value in 0..3 truncates to 25. (a) `myspeed` is compared, on widening windows only (`harness.rs:2078`). (b) Why nothing parted: the arm is the identity. Crate `rares.rs:253` `speed_of`. | none; one line on parked 3343a carrying the measurement (a citation) |

## Part 2

| va | name | kind | first | second | verdict | settles it | the item |
|---|---|---|---|---|---|---|---|
| 00607a60 | UnitData::is_seen | CLAIM | twin-differs (`unit_masks & 0x800`; `0x8000` never raised) | differs (same; "seen here") | **uncompared** — the state is masked out of the compare and already disagrees on a pinned window | `is_seen@00607a60:11–13`: `(unit_masks & 0x800) == 0 && (type+0x2b4 & 0x4000) == 0 && (unit_masks2 & 0x8000) == 0` else `is_detected` unless blown. Crate `fight.rs:1453–1466` reads `CLOAKED`, `unit_masks2 & 0x8000` and stealth-idle but not `0x800`; nothing in `crates/sim` writes `unit_masks2 & 0x8000` (grep of every `unit_masks2 \|=`: `0x4`, `0x4080`, `0x200`, `0x800`, `0x1000` only) nor `unit_masks & 0x800` (`cast.rs:1261` SEAM). (a) Both bits are printed and parsed (`gamelog.rs:3506–3507`) and masked off: `unit_masks2` is compared for `0x10\|0x200\|0x800\|0x20000` (`harness.rs:276–284`) and `0x40000`, `unit_masks` for seven other bits. **Measured** on run1228b (the Iroquois ancient pair's window 8165..8421): 18,560 unit-frames carry `unit_masks2 & 0x8000` and 66 carry `unit_masks & 0x800`, every one who=1, the Iroquois; the crate holds 0 on all of them. Also 35,508 carry `0x80` (1,024 of them the Scouts' `0x4080`), which the crate sets only on Scouts (`lib.rs:2557`) — a second masked gap, `process_cloak`'s input. (b) The consequence — an enemy refused a target — needs who=0 to choose among cloaked who=1 units it has not detected; on a click-free lobby who=0 rarely picks a target, which is why the draws held. | **a compare line on disk**: add `0x8000` and `0x80` to the `unit_masks2` mask and `0x800` to `unit_masks` — red at once on run1228b and run897 block 1; that widening is what books `process_cloak`'s port (parked 3114, 3115) |
| 00609c00 | UnitData::total_damage | CLAIM | twin-differs (no dead-figure share, no fraction) | differs (same; the `.max(1)` is the crate's own) | **confirmed** | `total_damage@00609c00`: captain sums `+0x24` down `o_down` (live only), plus `((uber_size − curr_uber_size(this,1))·hits)/uber_size`, plus `(param_1 + damage_frac) >> 4` when `0xf < sum`. Crate `lib.rs:5314–5319`: `Σ (max_health − health)` over `squad_members`, `.max(1)` on the result; `transport.rs:1092` `same_damage` reads one figure. (a) `damage`/`damage_frac`/`myhits` are compared per frame (`harness.rs:196–199`, `hits_diverged`). (b) The TSV's entering runs reach the function through `ObjectData::health_level` (UI and AI read it) — not the twin's consumer; the parting needs `gain_tech`'s shrink to `uber_size` 1 on a squad already short a figure or holding a fraction, or a boarding/conversion through `same_damage`, on a dumped frame — not on any compared window. | a capture of a squad-shrinking upgrade researched while a squad is short a figure (`U3` on the research frame: the captain's `damage`) |
| 0060fe90 | UnitData::max_range | CLAIM | twin-differs (Liberty, Eiffel, Ptolemy, Napoleon, Rum, Obsidian) | differs (same) | **confirmed for Rum alone**; the rest are inert on stock skirmish | `max_range@0060fe90:52–54` `(leader+0x6da8 & 0x20) \|\| (+0x6dd0 & 0x20)` → `+ rum_naval_range`; `:73–75` Obsidian. `Data/rules.xml`: `RUM_NAVAL_RANGE 1`, `OBSIDIAN_ARCHERS_RANGE 0`, `LIBERTY_SIEGE_RANGE 0`, `EIFFEL_SIEGE_RANGE 0` — three of the six add zero; Ptolemy (`0x16a`) and Napoleon (`0x167`) are gated on the leader's general counters `+0x59d2`/`+0x59cc`, which count units a skirmish cannot build (by name: CtW's generals; see Not established). Crate `fight.rs:545–549`, `:577–581` SEAMs. (a) No range is printed; the stand-off position and first-shot frame are compared per frame, through positions and `firing_diverged`. (b) Needs a ship of a Rum holder in a fight on a dumped frame; the leaders' rare bits print as `rares_collected[scan]` (run1179's LEADERDATA), never compared for this. | a dump grep on disk: a lockstep game's LEADERDATA holding Rum while its warships fight on a dumped frame; else a capture of a Rum-holding navy |
| 00610160 | UnitData::armor | CLAIM | twin-differs (seven named terms) | differs (same; The Citizen and The Monarch named as buildable) | **confirmed for The Citizen and The Monarch**; Ptolemy, Darius, Wellington, Blücher, Memnon gate on general counters a skirmish leaves at 0 | `armor@00610160`: inside `if (-1 < find_hero(..))` — `+ general_rally_armor·(upgrade+1)`, then `has_general(this,0,0x164)` → `+ thecitizen_armor_bonus` (rules.xml 2) and, on cavalry kind `type+0x40 == 0x1ac`, `has_general(this,0,0x162)` → `+ themonarch_cav_armor` (2). The Citizen is itself a hero to the crate (`group.rs:2277`: TY 356, `GENERAL`), so its own radius opens the gate. Crate `fight.rs:493–522` `rally_armor` stops after the rally term (SEAM `:501–503`). (a) The blow's result — `damage`/`damage_frac` — compared per frame. (b) `docs/PARKED.md:11674` (3677b): "no window on disk holds a blow on a unit beside a Citizen that parts"; `:11764` (3688c): "No blow on disk strikes a Stable unit by a Monarch". | a capture of a fight beside a Democracy player's Citizen, or a Monarchy player's cavalry beside its Monarch (`U3`, the struck unit's `damage` on the blow's frame) |
| 0063e4b0 | Wall::activate | CLAIM | twin-differs (Air Defense `job_counter = 0x40000000`); field compared only on an unfinished site | differs (same); field "printed and compared" | **uncompared** — **A** on the instrument: the gate excludes exactly the case | `Wall::activate@0063e4b0:45–54`: `job_counter = 0` then `if is(0x20b) job_counter = 0x40000000`. Crate `city.rs:2277–2340` writes 0 only. (a) `BuildDump job_counter` is off both pins, but its one compare is `b.job_counter.filter(\|_\| !ours.active && b.flags & 4 == 0)` (`harness.rs:687`), and activation sets `flags \|= 4` and `active` in the same call — so the Air Defense's value is never compared. B's "printed and compared" reads the key pin, not the gate. (b) On a finished building `job_counter` is the turret's angle: `Build::process@0061edf0:116–124` turns it toward the target, and no simulation reader was found (A's export grep of `field_0x48`; `Build::process` writes it after `do_attack`). Not a playing rule. | none: a widening would part on any capture holding a finished Air Defense and move no score |
| 0063fa60 | WallData::armor | CLAIM | twin-differs (Senator arm) | differs (same) | **confirmed** | `WallData::armor@0063fa60`: `leader+0x59c0 != 0` → `find_hero(x, y, who, 0, 0x161, (type+0x238 + +0x234)·0x60)`; found → `+ thesenator_build_armor` (rules.xml 5), before the `!is_active` halving. The first arm (`vslot 0x18 && is(0x161)`) needs vslot `+0x18`, which on `Build::vftable` is not a hero test (A). Crate `fight.rs:478–489` SEAM `:473–477`. (a) Building `damage`/`damage_frac` compared per frame (`harness.rs:706–711`). (b) Parked 1359: the captures that hold a Senator (East Indies `1/60`, run470 `1/80`) land no strike within its reach. | a capture of a Republic player's building struck within its Senator's reach (`BUILDS`, the building's `damage` on the blow) |
| 006b42c0 | WorldData::is_really_seen | CLAIM | unmodelled | missing | **confirmed** | `is_really_seen@006b42c0`: `p > 7` → 1; `leader_flags & 0x800` or `num_units[0x141]` → 1; `& 0x2000` and an allied cell owner → 1; else `seen[..] & ally_mask` — no `reveal_map` exit. Its one caller asks it of the tile's owner as `in_enemy_territory`'s last arm. Crate `place.rs:620–641` takes that arm false (parked 2250). (a) A refused placement shows as a building present on one side only (`build_unlinked`) and in `LEADERDATA` resources, both compared; the AI's site choice moves draws. (b) The function was never entered by any lockstep game (TSV `-`): it needs a placer with no city anywhere and none in the region. | a capture of a cityless player (every city lost) placing on a rival's seen ground; not taken by item 3992: a golden lobby defeats a cityless player, so the route is a Nomad lobby (parked 3992a, `docs/GOLDEN.md` §185) |

## Summary

Fourteen rows, all `CLAIM`, both readers agreeing the crate differs on every one.

| verdict | n | rows |
|---|---|---|
| confirmed | 11 | do_garrison, pitch_aircraft, check_fuel, think_carry_idle, think_carry_military, think_caravan, total_damage, max_range (Rum alone), UnitData::armor (Citizen, Monarch), WallData::armor, is_really_seen |
| uncompared | 2 | is_seen (masked bits), Wall::activate (gated key) |
| refuted as live — inert | 1 | update_speed (the Aztec arm truncates to the identity at the citizen's stock speed, measured) |
| not printed | 0 | — |

Reader disagreements inside the rows: Wall::activate — **A** (the compare is gated off the
finished site); do_garrison, pitch_aircraft, check_fuel, think_caravan — **B**'s wider lists stand
on the export, A's single delta is a subset of each; max_range and UnitData::armor — both list arms
that are inert on stock skirmish (zero constants, CtW generals), which neither marked.

**Why no pinned game has parted, in one line per class.** A lockstep game is pinned on its draw
stream; records are compared only on dumped blocks and the parting rows there are pinned counts.
So: is_really_seen and do_garrison — the function was never entered; think_carry_idle — entered,
and either refused or drew nothing (unmeasured); the other nine confirmed rows — a narrow case
(a flier struck, a home razed in flight, a dead target in transit, a route ending on a city's
death, a shrinking upgrade on a short squad, a Rum navy, a blow beside a Citizen/Monarch/Senator)
that no compared window holds; is_seen — the differing bits are masked out, and their consumer
needs a target choice who=0 rarely makes on a click-free lobby; Wall::activate — gated, and read by
no rule; update_speed — the identity.

**The items, cheapest first.**

1. **is_seen / process_cloak — a compare-mask line on dumps already on disk**: `unit_masks2` gains
   `0x8000` and `0x80`, `unit_masks` gains `0x800`; red at once on run1228b (18,560 + 35,508 + 66
   unit-frames) and run897 block 1. The widening that books `process_cloak` (parked 3114, 3115).
2. **think_carry_idle — a test run on disk**: `RON_FIRSTS=1` on the two melee word-window tests
   (run1483, run1492 stride); a boat's order row among the pinned `firsts` names the case, its
   absence says the spot test refused. Then wire `carry_idle.rs` (parked 3411a).
3. **update_speed — a citation**: parked 3343a amended with run1256b 8711's measurement
   (`myspeed` 25 on 51 Aztec gatherers; the constant loads 0..3; inert).
4. **max_range — a dump grep**: a lockstep game's `LEADERDATA` holding Rum while its warships
   fight on a dumped frame.
5. **think_caravan — a dump grep**: a lockstep closing window (run1017 15230..15433) with a city
   captured while its caravan held a route.
6. Captures, by lobby: a Citizen/Monarch fight (UnitData::armor), a Senator-covered building
   struck (WallData::armor), a squad-shrinking upgrade on a short squad (total_damage), a soldier
   barge whose target dies (think_carry_military), an Airport razed with its plane out
   (check_fuel), a Fighter striking a plane (pitch_aircraft), a Helicopter garrisoning an Airbase
   (do_garrison, with pitch's helicopter arm), a cityless placer (is_really_seen).
7. Wall::activate — none.

## Not established

- Whether Ptolemy (`0x16a`), Napoleon (`0x167`), Darius (`0x16b`), Wellington (`0x170`), Blücher
  (`0x175`) and Memnon (`0x16c`) can stand in an ordinary skirmish. Taken here, by name, as CtW's
  generals — so their `max_range`/`armor` terms are inert outside CtW; the unit table's buildable
  flags would settle it.
- `AZTEC_MOVE_SPEED`'s exact value: the measurement bounds it to 0..3 and does not separate them;
  any of the four is the identity at `myspeed` 25, and no stock Aztec citizen moves at another
  speed in the window read. A modded or upgraded citizen speed would part if it is non-zero.
- Whether think_carry_idle's two lockstep entries refused at the spot test or made an order that
  drew nothing (item 2 above). Function coverage cannot tell an arm.
- Whether any of check_fuel's six entering games razed an Airport with a plane aloft, or any game
  had a plane homed on a carrier: their `cover=1` dumps hold closing windows only; run1492's
  stride holds no air order on any of its 27 blocks.
- That who=0 picked no target among cloaked who=1 units in the Iroquois lockstep games: the reason
  given in is_seen (b) is the likely one, not a measured one.
- The spawn message named `TypeData::get_cost`'s Beaver Pelts arm, "no craft is ever priced" and
  `Unit::process_attrition`'s alliance exemption as on this list; they are on `adj-3.tsv`
  (`00664090`, `005e11a0`), not `adj-4.tsv`, and are not adjudicated here.
