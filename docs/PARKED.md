# Parked

The backlog that does not boot. `docs/QUEUE.md` holds what is in flight and
what is next; this holds everything else, and **a fresh session does not read
it** — the queue's opener is what starts a session, and this file is opened
when a *wave is composed*, which is a different and rarer moment (item 263,
decided 2026-09-07 with Ramon and lore).

Two rules ride with that, and they are the conditions the split was agreed
under:

- **Read this when composing a wave, not at boot.** Once per wave is cheap;
  once per session was the cost being removed.
- **A cross-item constraint does not park.** If an item says something a
  worker on *another* item must obey — that a file is about to be split,
  that a mechanism cannot move a scored word — it is stated in that
  worker's brief, or the item is promoted back to the queue. Two workers in
  one file is the failure this split would otherwise buy.

Numbers here are live: `tools/queueledger.py` reads this file alongside the
queue, so moving an item between the two is not a deletion and never reads
as one. Everything in `docs/QUEUE.md`'s "How to maintain this file" applies
here too, except the item cap — a parked item is not an open one.

## Parked by item 718, 2026-09-24 — the garrison's other arms

(724) **`action_garrison`'s unbuilt arms**: `QUEUE_FIRST`, the editor, a
worker's `QUEUE_FIRST` (the decompile's registers are garbage there; the
listing settles it), packing types, and `search` from the command; and
`action_eject_all`'s `back_to_work` filter and `eject_o`/`eject_who` arm.

(725) **`GarrisonOrder::get_garrison_order@00482dd0`'s vtable slot.** A PE
read of the vtable would cite it and move the census's order row 49 → 50.

## Parked by item 714, 2026-09-24 — the follow's other arms

(719) **A follow on an enemy unit**: `action_follow` has no `is_ally`
test, and `is_seen`'s fog or stealth kill would end it. No capture
stages one.

(720) **The follow's container swap** (`oxx`, `whose`, `uid2`), read and
not reached by run204.

(721) **`action_follow`'s `QUEUE_FIRST` insert**, read and not reached.

## Parked by item 711, 2026-09-24 — under Great Lakes' 15383

(716) **`1/79`'s move `facing`**, 1 here against 0 in the original from
15351: a value row, no draw.

(717) **The mirror bit's two unwired readers**: `set_new_location` at
`5f9290` and `Guy::set_anim` at `5dafad` read `unit_masks & 2`, and this
crate carries the bit now (GROUPS §25) but neither reader.

## Parked by item 706, 2026-09-24 — the patriot arm's seams

(712) **Three seams of the Senate's patriot** (`docs/TECH.md`, "The
government patriot"): `set_type` on a second government, the respawn
(`Unit::close`, `Build::process`), and `Build::activate`'s Senate head.
No capture reaches any of them. 679 stands as a second cause: the graft
table did not move the citizens.

## Parked by item 707, 2026-09-24 — the guard's listing-only arms

(710) **The AI guard's roll at `fight+0x824`, `k` = 3, the captain
shortcut, and the retaliation's `on_duty` return** (COMBAT §63.6): read
from the listing, reached by no capture.

## Parked by item 696, 2026-09-24 — past chapter eleven's 734

(704) **§4.3's first soft row, two caravans** (`TRADE_ROUTE` both ways,
both moving), is still not carried in `sim::collide` (COLLISION §4.3).

## Parked by item 695, 2026-09-24 — under Great Lakes' 14650

(699) **who=1's `known_rares`**: ours 4 against 0 on 14536, with or
without the Senate arm.

(700) **`senates_built`**: the leader record prints it (0 → 1 on 14529)
and nothing in this crate reads it.

(701) **The lazy border recompute after `fix_borders`**: who=0's
`gather_stamp` is 14536 here against 14544 in the original.

(702) **`TerrainOut::caravan_step`'s camel steps and `element_num`**, the
stray-road sweep's two unmodelled inputs (ROADS §10.5).

## Parked by item 676, 2026-09-24 — chapter nine's other edges

(689) **A pushed group's `GroupData::id`**: ours `64 +` its index in
`Sim::pushed`, the original `who·64 +` the pool slot; run180 block 642
reads 641000 against 647600, and chapter ten's patrol ids the same
(693); 718's `come_out` squad push and the eject's building group are
pool slots of the same family. No step reads it, but the fix reaches the
AI's pushed groups on both long captures, so it wants both words
measured. Likely 674's cause (an order's group id).

(690) **`input::Stream`'s recorded `MoveTo`** still gives each unit
`add_move_order` and bypasses the group; `input::group_move_to` is the
original's entry and the recorded stream should take it. run7's tests
pin the stream's counts.

(691) **`UnitData::play` (`+0xb6`) is printed and pinned unread.**
`process_group` writes it on every commanded unit; run180 prints `play 0`.

## Parked by item 680, 2026-09-24 — chapter six-b's other arm

(686) **§62.4's unit-target arm.** The original's `local_14` search is
`find_new_target`, a kill and a fresh order, where this crate retargets
in place (COMBAT §62). It names no frame yet.

## Parked by item 651, 2026-09-24 — past chapter six-b's 632

(681) **`Built::build_ids` names no building placed after `BEGIN GAME`**,
so a staged Airbase reads `None` as an attack's `order:target`: item
462's shape one level over. A fallback to `(owner, index)` removes both
rows and moves nothing else; it reaches the long captures' widenings.

(683) **Whether an aircraft on the ground may strike a building at all**:
the Fighter is `ANTI_AIR`, and neither aircraft does in 640 frames.

(684) **The air line that flies still has no staging.** A trained
aircraft inside its base, or an issuer (`issue_launch_patrol`,
`issue_flight`), is the only writer of a player's air order: the issuer
axis (GOLDEN §13).

## Parked by item 673, 2026-09-23 — under Great Lakes' 12897

(679) **who=1's citizens hold `myhits` 40 against 50 and `mylos` 2
against 4 from block 12564**, and the newborn `1/69` on 12566 the same.
No draw; it reads as an upgrade the original applies and this crate does
not (`docs/journal/2026-09-23-item-673.md`).

## Parked by item 669, 2026-09-23 — under Great Lakes' 12536

(674) **An order's group id**: ours 12280205 against 12286605 on 12281,
again on 12537. No draw.

(675) **Birds are owner 9 and no dump prints them**, so their rolls cannot
be widened; a bird's draw on a word's frame is readable only in the draw
stream or a packet. And `docs/RUNS.md`'s run163 check table sits inside
run169's section, a union-merge artifact.

## Parked by item 668, 2026-09-23 — past chapter eight's end

(671) **The meeting on a first blow.** Both leaders' `treaties[·]` read 3
in run171 from block 660, the first blow's block, and 0 here: bits 0 and
1 are set on the blow, and this crate's only writer is VISION §6.2's
meet loop. No draw.

(672) **`fight`'s retarget budget**, `waiting < 5 && retargets > 10`
(`LAB_005fdb9e`'s first arm). The counter is carried now; the arm is
not, and no capture is known to reach ten in a frame.

## Parked by item 660, 2026-09-23 — chapter eight's other halves

(665) **A fight across an alliance** needs a third live leader: in a
two-player lobby `ally 1` is an allied victory (`set_diplo` counts
leaders 0..7), and run171 ends on 901. A new lobby, not a restage.

(666) **The rally armor's size**: one point here where `rules.xml` reads
2. Read on the blow, past the word.

## Parked by item 657, 2026-09-23 — the army's other edges

(662) **`go_to_unit`'s walk group 68 is freed on 12086 in the original**
and kept here (`pool:68`); no draw.

(663) **`find_target`'s head test `2 < epoch[0] && type_avail(SUPPLYWAGON)`**,
which this crate's `weak_army` does not carry. No word names it.

## Parked by item 571, 2026-09-23 — under Great Lakes' 12135

(658) **`1/41`'s idle variant on 11922**: a roll assigned to another figure
on one block; no draw.

## Parked by item 650, 2026-09-23 — the target checks' other arms

(654) **`valid_target`'s helicopter and `is(0x132)` arms**, not carried
(COMBAT §61.5). No capture stages a helicopter.

(655) **`check_target`'s region and `DEFENSIVE` gates in the ring search**
(§61.5): 650 made the ring search call `poor_target` as `check_target`'s
tail does, and left these two out. No word names them.

## Parked by item 652, 2026-09-23 — the bird in flight

(659) **A bird's unit `+0x78` is its birth tile and never moves in
flight.** It names no score until an air unit is dumped (651).

## Parked by item 648, 2026-09-23 — chapter six's other halves

## Parked by item 642, 2026-09-23 — under East Indies' 13640

(646) **run159's five rows from block 11793**, none spending a draw before
the word: the boat `1/44`'s `form` −1 against 0 (the Merchant's `form`
residue); its order's `facing` 0 against 1, inherited from the scout's
first-block row; the scout's `avg_speed` 25/24 against 18/18 and its
`stopped[1]` 0 against 1. 657 read the `form` residue: `Unit::init@00612100`
writes `+0xaa` as 9 for types `0x32`–`0x35` and 0 otherwise, and `+0xab`
as −1; building it re-pins `form` rows across a dozen widenings. The human's census still does not run
(`docs/AI.md` §23.1): the first-block `0/-1` leader and `0/2000` city rows
stand.
698 found the same shape on Great Lakes: three newborns' `form` −1
against 0 on 14946 (`docs/journal/2026-09-24-item-698.md`). 706 found it on the
Senate's patriots too: `1/79` on 14983 and `1/60` on 15783.

## Parked by item 627, 2026-09-23 — the search gate's other arms

(640) **A building's arm of `find_nearby_target`'s reach gate** (COMBAT
§60.5): from the listing, a building sets `local_24` with `local_5c` clear
and skips every out-of-range candidate; this crate keeps one with
`in_range = false`. Any capture where a tower or a town centre searches
measures it.

(641) **`local_5c`, a computer's packed siege engine's exception** to the
same gate (§60.5), not carried. No capture stages one.

## Parked by item 617, 2026-09-23 — under the restage's 865

(635) **The scout `1/0`'s fresh `EXPLORETOORDER` move on 847**: `facing 1`
here, 0 in the dump. No draw.

(636) **The catapult's round takes pool slot 0 here and 1 in the dump on
798**, run145's family (COMBAT §55.5). No draw.

(637) **The death object's cull** (COMBAT §59.7): `DeathObj::inc_time`
clears it when its packet ends, and this crate never does; no window
reaches one, and 617's hold is exact on every window that exists.

## Parked by item 628, 2026-09-23 — the cheat's two terms

(634) **The `ai_off` stand-in's corrected predicate is `!ai_driven(owner)`**
(`docs/INPUT.md` §11.10): `orders.rs`'s `ai_off && !ai_driven(owner)` was
derived on the retired reading of bit 4. For who=1 the two agree, which is
why neither capture parts there. For who=0 without the cheat, ours opens
`Unit::think`'s tail where the original closes it (§11.9's seam). The fix is
one term, and it wants its own gate against the long captures, whose who=0
is the human. Returns when a word names who=0's tail.

## Parked by item 621, 2026-09-23 — past the restage's word

(626) **The catapult's round leaves the unit's square**: `sim::launch` has
no node for piece 265's release (COMBAT §22's seam, beside (611)).

## Parked by item 613, 2026-09-23 — under East Indies' 11069

(622) **The keys under the word on run152**, none on 11747's draw: the
peasants and `1/36`'s and `1/41`–`1/43`'s hit points
(`docs/journal/2026-09-23-item-613.md`); `SITE` on 10976 closed on 688's
halving (AI §67). `1/2018`'s list order closed on
620's clamp, and food and metal a unit off on 629's Merchant seat.

(624) **The rest of `get_cost`'s pre-ramp tail and its context
argument**, still unbuilt beside (555)'s bump loop. 613 built the Horses
discount only. 629's Merchant seat took run123's and run117's `MAKE`
value rows off it; whether any of `get_cost` itself is left is unread.

## Parked by item 602, 2026-09-23 — under a closed chapter and its restage

(618) **run145's 729/753: the turned chariot's turret.** The dump needs a
1° step where the node bearing gives 0°. A `GUYS=4` capture over
705–760 answers it. run145 is closed at 900, so it names no score.

(619) **A round's target is cleared at death here, at the due frame in
the original**, and the ammo pool's slots are one apart. Neither draws.

## Parked by item 608, 2026-09-23 — the slot count's edges

(615) **`count_gather_slots`' `open` floor.** This crate floors each
building's share at 0, and the original has no floor. No row shows it
yet.

(577), partly paid by 608: `widen_east_indies` now compares gaia's
position and clock; the general `compare` still skips gaia.

## Parked by item 603, 2026-09-23 — the other pivots

(611) **Node offsets for every other pivot piece** (`docs/COMBAT.md`
§54.5). 603 established the Chariot's (piece 145, node 4) on the packet
and under unicorn; the rest are read, not run. Returns when another
restricted type's verdict parts.

## Parked by item 604, 2026-09-23 — the templates' unchecked half

(609) **The XML-order-to-slot mapping of the mountain templates** rests on
a reading: the packet confirms it on 3 of 16 templates
(`docs/FORMATS.md`, "The mountain templates"). Returns when a range on
another template parts.

(610) **Great Lakes' solid cells** come from the same loader and no packet
has checked them. The placed tiles equal the start dump's on both maps;
the solid lists are unverified there. Returns when a Great Lakes row
names a Mine or a mountain.

## Parked by item 597, 2026-09-23 — the placement's reading-only offsets

(607) **A blind second reading of `docs/AI.md` §59.6's offsets.** The
Mine's site is diff-backed by run143 and the packet; the offsets no run
reaches rest on one reading.

## Parked by item 595, 2026-09-23 — the pivot's last degree

(603) booked 2026-09-23: 601 moved chapter three's word to 684, where
the verdict parts at the boundary.

## Parked by item 592, 2026-09-23 — the rider's other seams

(599) **`docs/AI.md` §58.5's four seams**: the sea rider, the region-less
move point, the on-map `8`/`0xe` arm's use of the same local, and the
`is_move` classes. None is on 10582's frame.

(600) **A blind second reading of `docs/AI.md` §58.1**, the container
count. The census on 10576 is diff-backed; the arms no capture reaches
rest on one reading.

## Parked by item 588, 2026-09-23 — the probe's other half

(593) **The land half of the waypoint probe's second arm** (siege, hero,
supply; `docs/COLLISION.md` §13.5), and **the pushed unit's guy turn**.
588 built the sea half only; no word's frame reaches the land half yet.

(594) **A blind second reading of `detect_boat_collision`.** 588's sea
half rests on one reading and the listing; the parts no capture reaches
are reading-only and owe a second reader under CLAUDE.md's audit rule.
Returns with (593) or when a sea word's frame reaches them.

## Parked by item 587, 2026-09-23 — what `age` hands over

(591) **`age`'s knowledge and metal buckets**: 100 in the original after
`age who=N 2`, 0 here, on both of chapter three's captures from 605. No
draw depends on it in either. Returns when a chapter or a long capture
spends knowledge or metal after an `age` line.

## Parked by item 579, 2026-09-23 — the navy's unbuilt corners

(589) **Four corners of 579's fix, none on a word's frame**
(`docs/ORDERS.md` §25.5): the aircraft-carrier half of the warship
predicate, which no capture reaches; `SEARCH_FRIENDLY`'s allies in the
navy's `find_city`; `remask_docks`' candidate set; and the dock
margin's pathfinder cost. Each returns when a row names it.

## Parked by item 578, 2026-09-23 — a seam no run exercises

(586) **Nubia's "50% more" hit points on merchants, caravans and markets
is unapplied** (`NUBIAN_HIT_POINTS`, read by `Unit::update_hits@0060e930`).
In chapter seven the Caravan, Merchant and Fur Trapper carry `myhits`
135/135/180 in the original against 90/90/120 here; pinned under the
word, and no draw depends on it there. Great Lakes' player 0 is Nubia
too, so it returns as its own item when a Great Lakes row names it.

(585) **`ai_off && !ai_driven` is folded in `sim/orders.rs`**, and it lets
a human's units into `Unit::think`'s tail with the cheat off, where the
original's gate reads `leader_flags & 4` (`docs/INPUT.md` §11.9, SEAM).
No capture exercises it: every golden chapter runs a human with the cheat
on. Returns when a chapter or a long capture runs a human without it.

## Parked by item 576, 2026-09-23 — under East Indies' word

(580) **The Citizens' hit points and line of sight on 10165**: 40 against
50, and 2 against 4. Under the word; spends no draw on 10232.

(581) **`gather_stamp` on run139's 9992**: ours 9991, the original's 9719.
A stamp that is written on a gather and never cleared, so it asserts a
change and not a value (QUEUE's upkeep rules). Under the word.

(582) closed 2026-09-23 by item 579: the tile walk gives the navy's
point and was on the cause chain (`docs/ORDERS.md` §25).

## Parked by item 573, 2026-09-23 — the player nobody compares

(577) **`compare` never reads gaia's units.** It walks players
`0..players`, so `who 8` is outside every row. On East Indies 9984 the
animal `8/0` changes `cur_anim` 0 → 2 on one side only and parts on no
row (`docs/journal/2026-09-23-item-573.md`, "9983"). A gaia figure that
parts is invisible until it touches a player's unit. Returns when a
word's widening needs a gaia row, or with the coverage pin.

(342) re-measured by 573: all six scholar seatings in East Indies'
[7880, 10399] are on the original's University on the original's frame.
Host choice seats no Scholar wrongly in that window.

## Parked by the eleventh Fable pass, 2026-09-23 — a reading, off the Loop

(526) **Sweep every `find_angle`/`sinx`/`cosx` call site `docs/` cites
against the listing.** Filed by item 523. Three of three checked on
2026-09-22 had been misread: `compare_target`'s `find_angle(0, 0)` (495),
a landed shot's `find_angle(num_guys, index)` and the lead's speed (523).
All three were register-passed pairs the decompiler dropped; the trap is
now in `tools/ghidra/README.md`. The sweep is mechanical and bounded, and
every uncorrected site is a claim two sections may be carrying as fact.
Off the steering pass's list (DECISIONS 47): a listing sweep is a
reading and names no score. The trap is in `tools/ghidra/README.md`,
and a lane whose item touches a trig site checks the pair; the sweep
returns when a word lands on one.

## Parked by item 566, 2026-09-23 — the memo's twin

(572) **The pathfinder's copy tree (`+0x4c`) has the memo's lifetime and
is still a seam** (`docs/PATHFINDER.md` §24.5). 566 carried the validity
memo (`+0x50`) across searches until `kill_lists`; the copy tree lives
the same way and this crate still rebuilds it per search. Not 12038's
cause; beside (546), the region gate's `nocoll` arm that reads it.

## Parked by item 552, 2026-09-23 — the namesake's unwired edges

(568) **The budgeted border sweep.** `GameDaemon::check_borders` spends 256
cells a frame after a five-block delay; observed on all three of chapter
four's levers, not modelled. It is run132's only border parting (301, 30
cells a sweep early) and spends no draw.

(569) booked 2026-09-23: 567 moved chapter four's word to 1416, and its
widening names the wagon.

(570) **`calc_anti_attrition` is unwired** (Foraging, Mongols, Titanium,
Liberty): every resistance is the base. And **`Unit::squad_size` is a
stored 1** that `fight.rs` reads for "alone" — a question for combat.
Promote either when a capture needs it.

## Parked by item 560, 2026-09-22 — a sweep nobody probed

(564) **Which group-mate `1/37`'s collision sweep passed on 11804.** No
`RON_COLLIDE_PROBE` build covers the frame, and 560's fix (the soft
one-shot is set only when the sweep ends without a hard hit,
`docs/COLLISION.md` §12) does not depend on it. No score names it.

## Parked by item 557, 2026-09-22 — the pool's other writers

(561) **`Unit::come_out`'s own push of its squad** (group 69 on 11423),
the only residue of run134's pool dump; the list it leaves at 11424's
add is the same either way. Pinned in `run134_s_pool_list_is_the_original_s`.

(562) **Five unmodelled writers of `+0x80 = −1`**: `do_build`,
`build_done`, `do_attack`, `do_non_flat_gather` and `think_peasant`. They
could renumber the early-game pool (`docs/GROUPS.md` §19.4, run69's
scout slot churn). No score names them.

## Parked by item 554, 2026-09-22 — the group order's identity

(558) **The group order's `id` carries the army slot, not the pool
index**: 2 here against 66 in the original. `group_id`'s comment says it
is compared nowhere, but `order:group.id` parts on run130's widening from
11513 (`1/60`). A value row with no draw; promote it with 557 if the
pool membership fix reaches it.

## Parked by item 545, 2026-09-22 — `get_cost`'s unbuilt corners

(555) **`get_cost`'s available-arm bump loop** (`get_cost:159`–`205`),
reachable now that this crate queues upgrades, and the research arm's
Wine, `SPECIAL_UPGRADE`, American and Dutch terms. 545 built the military
discount and the research arm's base (`docs/AI.md` §56); these are the
rest, read and not built. No score names them.

(556) **`MAKE[3].val` 38,500 against 37,500 on run123's 11185**, the
Mine's value. A value row with no draw on the word's window; promote it
when a word lands on a Mine offer.

## Parked by item 549, 2026-09-22 — the attack slots a packet lacks

(553) **A census of graphic packets with fewer than three attack slots.**
`set_anim`'s attack arm plays `CHAR_ATTACK2` when the packet lacks the
rolled slot (`docs/ANIM.md` §4.13), so every such attacker swings
`ATTACK2` on a third of its rolls; and a packet lacking `CHAR_ATTACK2`
too gets 3 frames, which no capture has shown. A census over the
install's packets would say which units are exposed; no score names it.

## Parked by item 543, 2026-09-22 — the rare-collector arm's other halves

(551) **`unit_masks & 0x100`, "ordered recently", is not kept**, and the
arm's `idle += 1` path, which no capture exercises. Both are the same
arm's; promote with (550) or when a word lands on an idle human unit.

## Parked by item 542, 2026-09-22 — the keel, and the other releases

(547) **The trireme's keel nodes at a second facing.** 542 measured the
release point as node 0 on the keel from three rows of run127, all on
one bearing. A trireme engagement on a different bearing (`AMMO=5`) is
what would confirm the node rotates with the hull rather than fitting
one facing. No score names it.

(548) **The other 355 separating events.** `GraphicEvents::init_unit_events`'
release frame is `starttime/67` floored at 1 (542, `docs/COMBAT.md`
§50); only the trireme's is measured. Crossbowmen first, when a capture
reaches them.

## Parked by item 539, 2026-09-22 — a seam only a second region shows

(546) **The probe-side region gate's `nocoll` arm** in `collide.rs` reads
the pathfinder's `+0x4c` block copies, which can go stale across
regions. East Indies is the only map with more than one region, and no
score names it; promote it when an East Indies word lands on a
collision probe that crosses a region line. `docs/COLLISION.md` §4.2.

## Parked by item 535, 2026-09-22 — the first ships

(544) **`AMMO` in chapter two's widening.** `chapter_two_s_word_frame_is_widened_whole`
does not compare the `AMMO` record, which chapter five's widening now
does in a few lines. Chapter two is closed at 900, so it names no score;
it is the instrument half of (525).

## Parked by item 530, 2026-09-22 — chapter one closed, and what it left

(536) **A marching army group never looks around.** `do_group_attack_to`'s
look and `find_nearby_target`'s `Group::action_attack` arm for a
`GROUP_ATTACK_TO` (`docs/ORDERS.md` §22.5) are unimplemented. 530 names it
the likeliest of its three to surface on the long captures; the AI track
should check it first when a word lands on an army's march.

(537) **The army group's `speed`/`new_speed` at birth** (`Group::add` →
`compute_speed`): 0 against 25 on chapter one's 616. A pinned row of the
chapter's widening; spends no draw.

(538) **The guy's aim at stand-up**, from the dump's `GUY ox/whom`: owed
the day a capture opens mid-fight. No window on disk does.

## Parked by item 327, 2026-09-22 — the second writer

(534) **`compute_reg_territory` is a second writer of `reg_known_rares`**,
and this crate's zero there is `docs/AI.md` §55.3's lag: it accounts for
run91's one `known_rares` row (two `MAKE[0]` rows traded for it) and
names no score. Promote it when a word's widening lands on a
`known_rares` row.

## Parked by item 445, 2026-09-22 — chapter one's widening floor

(532) **Chapter one's standing floor rows**, none spending a draw in
`WIDENING_CHAPTER_ONE`: `0/2000`'s city record empty at 605, `form` −1
against 0 on every unit, followers' `orders_x/y` at birth seeded from
the captain, three citizens' idle `end_time` 33 against 56, and five
`dest_angle`s. Pinned by the widening; promote a row when a word lands on it.

## Parked by item 523, 2026-09-22 — chapter two closed, and what it left

(524) **`near_o` on `0/7` and `0/8` parts from 847**, pinned in
`NEAR_PARTED`. On 846 the bowmen drop their attack on the dead `1/7`; the
original keeps `near_o 7 near_who 1` and this crate clears to −1. Not the
search throttle (`waiting` is 0). It spends no draw through 900, the
trace's end, so it names no score. No mechanism named.

(525) **run112's `AMMO` records are compared against nothing this crate
simulates.** `diff::ammo` does it field for field for run109; run112 has
no such comparison, and 495's arc was checked on its three rolled shots
by hand. An instrument; promote it when a golden word parts on a shot.

## Parked by item 495, 2026-09-22 — past the word, and the remedy for a guard

(521) **A spent arrow holds its pool slot for 200 frames; this crate drops
it.** `Ammo::do_damage@00678060`'s puncture arm leaves a sticking shot as
`flags 1` with `cur_time` reset, and `inc_time` keeps it in its slot for
200 frames. run112 has 239 such records, the first on 780, past chapter
two's word (762). It names no score today; promote it when a word reaches
780 or a slot index is found to part on it.

(522) **run122: `master_land_heights` as raw bits.** An in-process read of
`TerrainData+0x464` at frame 0 settles all 2,058 of Great Lakes' ambiguous
height corners (`docs/COMBAT.md` §46.7). It is the remedy for the day
`Sim::ground_inexact` (`GROUND_INEXACT`) goes nonzero on any replayed
window; until then it buys nothing. The run number stays reserved.

## Parked by item 515, 2026-09-22 — a field the dump does not print

(519) **`march` (+0x4b) is owed, and no capture on disk can settle it.**
`GroupData::log_data` does not print it, so item 515 modelled the group
speed cap's `march = 0` write from the listing alone and **recorded it as
owed rather than guessed** — the right call, and the reason §18 states it.
Settling it needs a game with a **general**, which no capture on this disk
has. Promote it when a general appears in a booked capture, or when a
mechanic that reads `march` is worked deliberately; it is not a residue
chase and it moves no word today.

## Parked by item 510, 2026-09-22 — ten units an age too young

(516) **`g.gpiece` on ten pre-existing units at block 606** — `0/1..0/5`
and `1/1..1/5`, five types, both players. This crate's piece is exactly
`PIECES_PER_AGE` (**0x840**) below the dump's on every one: **one age
bracket**, not a scatter. `Sim::unit_gpiece`'s `(0..=bracket).rev()` walk
settles an age lower than `get_unit_gpiece@0090c030` does for this game.

**Costs no draw here** — `cur_anim`, `cur_time` and `end_time` agree on
all ten for all 123 blocks, and chapter two's nine staged figures are
unaffected. It is `docs/ANIM.md`'s, which was fenced to another lane, so
510 **pinned the rows in the widening's map and wrote nothing in ANIM**;
the write-up is owed. Parks because it names no frame and moves no draw
— but a uniform, exact 0x840 on ten units of five types across both
players is a single wrong decision, not residue, and it should be cheap.

## Parked by item 506, 2026-09-22 — a standing gap and two the ceiling found

**Booked as run121**: `MISC,LEADERS=2` over `[0, 10400)`. run60 did 5,400
blocks at that detail in under five minutes and 67 MB, so ~10 minutes and
~135 MB here. It converts an unbounded search into a lookup, because
whatever frame it names, run97 and run100 already cover in full detail —
and it is reusable, being the whole economy trajectory of the headline
game, which nothing on this disk has. **Take it behind the headline, not
ahead of it** (item 506's own judgement, and right: 10817 names its frame
and this does not).

**Disk caution, 2026-09-22: `/System/Volumes/Data` is at 99%, 15 GiB
free of 926** (re-measured by the commander; item 506 read 98% / 18 GB
minutes earlier, so it is *moving*). 135 MB is nothing on its own; a
queue of captures is not, and `ccc spawn` refuses below a 10 GB floor —
which is four run121s away. **Somebody should look at this before the
next wave**, and it is the kind of thing that stops the loop dead rather
than slowing it.

(511) **`1/2018 queue[0].cost[0]` on 10782.** Both sides now queue Horse
Archers at the Stable; this crate charges 60 timber / 40 wealth where the
original charges 57 / 38. A **price**, not a ledger.

(512) **`1/28 order:coll` on 10618.**

Both came under the comparison when the widening's ceiling followed the
word 10595 → 10830, and both are named in the test rather than swallowed
into a count. **A number that rises when a word moves is not a
regression** — same rule as (504)/(505).

## Parked by item 497, 2026-09-22 — brought under the line by the word itself

Item 497 moved Great Lakes 10303 → **10582**, and the widening's window
runs *to* the word — so 279 blocks nobody had ever compared came into
scope in one landing. Two counts went **up** as a result and **neither is
497's**: they were standing there the whole time, under a line that had
not reached them.

(504) **`1/35`'s standing position residue, from block 10353.** The
`run100_s_word_frame_is_the_original_s` position residue goes 5 units → 6.

(505) **`0/2000`'s `free`, from block 10400.** The same test's standing
city residue goes 20 fields → 21.

**Both are named in the test** — `RAIDER_SHORT` and `CITY_FREE` — rather
than swallowed into a count, so the day either closes, the pin fails and
says so. They park because neither names the headline's frame: 10583 is a
production frame and these are a raider's position and a city's field,
250 and 180 blocks under it. **A number that rises when a word moves is
not a regression**; promote either only when a takes-chain names it.

## Parked by item 496, 2026-09-22 — a piece that cannot score alone

(501) **`Sim::forget` drops a dead object from every attacker's target
slot on the frame it dies; the original keeps it.** Frame **684**, `order
0/6`/`0/7`/`0/8`, ours `None` theirs `(1,8)`, standing to 695. Invisible
until 496 widened `compare_orders`, and then thirty-six unit-frames of it,
under the word, quiet.

**Why this parks rather than queues:** measured, it moves no word alone —
the word stays 695 and two green pins go red. It is the smallest and least
informative of `docs/COMBAT.md` §43.2's four pieces, and the arm only
scores whole. It is therefore **inside item 502**, not a rival to it; do
not run it on its own on the strength of being true.

## Parked by item 494, 2026-09-22 — the wait table's unlit halves

Item 494 read `LeaderOptions +0x8` as an **index into five waits**
(1→7, 2→0xc, 3→0x11, 4→0x20, 5→0x3e, default→2) where this crate used the
index itself; `init` writes 2 and the wait is therefore 12.
`docs/ORDERS.md` §21. Three parts of that reading no run on disk reaches:

(498) **`think_caravan`'s half of the same table is reading-only.** Moved
to the shared `LeaderOptions::idle_wait` because the two listings are
identical — which is an argument, not a measurement. **No capture on disk
has a human caravan**, so nothing scores it. It is the honest kind of
reading-only claim: stated, not hidden, and `docs/ORDERS.md` §21's
coverage section says so.

(499) **`peasants_wait` is read from run12's `LEADEROPTION` record and
assumed to hold for run100**, which does not enable that category. The
whole of 494's nine-frame word rests on the assumption, and **one
`gamelog.ini` line on a Great Lakes capture settles it outright** — this
is the cheapest open question on file and the second genuine capture
question in twenty-odd landings (the first is (490)). ~~Promote it the
moment a Great Lakes capture is booked for any reason.~~

**Closed 2026-09-22 by item 506, off the disk and for two greps — no
capture was needed at all.**
`gamelog-run34-greatlakes-dumpall-start.txt` is this same game (map 14,
seed 12345, `GAME INFO` identical to run53's and run100's) and prints
`peasants_wait 2` on all eight leader-option records; and
`report.py rontrace-run53.log functions` puts
`CommandPackage::process_leader_options` at frame −1 and never finds
`set_auto_peasant_level` among the 6,937 functions the 24,000-frame
trace enters — so nothing rewrites it mid-game. `docs/ORDERS.md` §21 is
struck and points at both. **The fourth time in one day** that the disk
already held an answer something had booked a capture for; see (508).

(500) **The gate's `(idle − 2) % 5` retry above the threshold is
unexercised.** 494's citizen re-tasks on its first pass of 12, so nothing
in the window measures a retry at all. A second reading or a longer window
would reach it; the residue chase will not.

## Parked by item 489's landing, 2026-09-22 — its takes-chain target closed

(477) **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175), with
`g.x[0]` and `g.des_x[0]` the same — one world unit in x, said three times.
`docs/ORDERS.md` §18.3. It stood in the queue on one clause: that `1/28` is
the word's own squad and the row is **a takes-chain candidate to 10278's
collision**. Item 489 closed 10278 — `1/40` and `1/41` carry no row of any
kind in `[10270, 10307]` — so the chain has nothing left to reach, and the
new word (10294) is the **human's** citizen `0/5`, not the AI's squad.

The rows themselves are unchanged and still pinned, by value, in
`run100_s_word_block_is_every_record_the_dump_carries` ("block 10235 is not
item 477's three rows") — parking loses no measurement. It returns when
something names it: a takes-chain to a live word, or a floor that moves
with it. ~~**Do not re-attach it to 10294 on the strength of
proximity**~~ — that warning was right and remains right; proximity was
never the reason it came back.

**Closed 2026-09-22 by item 515: it was the word, through a chain nobody
had measured.** Block 10818's eight rows read as a collision and were a
*consequence* — this crate's `1/28` stood 370 units east, about fifteen
frames of its walk, so it met a gaia animal that sits in the same place
on both sides and never moves. `pos` was absent from the word's own block
only because it had **first parted at 10235**, which is this entry. One
step size: `UnitData::get_speed@00608720`'s last arm, the group speed
cap. 10235's three rows are gone and `1/28`'s position error is 370 units
→ **one**.

**The lesson is not "the park was wrong".** The park was correct on the
evidence then — its stated takes-chain had been closed by item 489 and
nothing named it. What returned it was a **measurement** of the chain,
not an argument from adjacency, which is exactly the standard this entry
demanded. A parked item is a claim about what is *known*, not a verdict.

## Parked 2026-09-21, item 471's successor

(474) **A recycled group slot keeps its `order_num`, and this crate's
fresh one does not.** Great Lakes' probe group carries order `id 8192502`
— group 65, frame 8186, **order_num 2** — where this crate writes
`8192801`, group 68, the same frame, **order_num 1**. Two reasons, both
read off the listing and neither modelled:
`Groups::copy_group@006fa690` copies `who`, `num`, `ox`, `oy`, `o_dist`
and `o_angle` into the pool slot and the four member arrays with them, but
**not `order_num`**; and `Groups::push_group@0070f9e0` skips `copy_group`
altogether when `Group::equals_group` matches the pool's *last* slot
(`groups +0x1c`), so a group pushed twice running reuses its seat with
everything on it intact. Either would leave a recycled slot counting on
from where it was.

**Why this parks rather than queues, and the clause matters:** it changes
the group order's `id` and nothing else any comparison reads. `id` is not
compared — the seat numbers are this crate's own and deliberately
uncompared (`docs/ORDERS.md` §16.6, and `Sim::group_id`'s comment says
so) — so closing this moves no word, no floor and no endpoint count. It
is on the record because it is *known* and because the `id` arithmetic is
how frame 8186 was established twice over; it should not be promoted to
the queue on the strength of being true. Promote it only if something
later starts comparing the seat, or if `order_num` turns out to be read by
a mechanism that is scored.

## Loop — the steering pass's, never a worker's

Tooling, guards and the queue's own rules. The Fable pass takes these
(`CLAUDE.md`, "Fan-out rules"); a commander never spawns one. Item 279's
two ledger regexes closed in the third pass, 2026-09-17; the fourth pass,
2026-09-18, ruled six (`docs/audit/2026-09-18-fable-pass-4.md`); the sixth,
2026-09-19, ruled ten — 377, 398, 406, 404, 411, 397, 374, 343, 356, 321 —
each now a guard or a clause (`docs/audit/2026-09-19-fable-pass-6.md`,
DECISIONS 42); the seventh, 2026-09-21, ruled nine — 416, 417, 419, 420,
421, 424, 431, 433, 436 — two guards, six clauses and a DECISIONS
paragraph — and **built 367**, the AI's dump, the same day: run114 and
`docs/AI.md` §52 (`docs/audit/2026-09-21-fable-pass-7.md`, DECISIONS 43);
the eighth, the same evening, ruled eight — 251, 335, 428, 446, 449, 452,
453, 461 — code, five guards, a lock, a fixture and a clause, each made to
fail first, and merged PR #4 whole (458); 375 left for the ordinary list
(`docs/audit/2026-09-21-fable-pass-8.md`, DECISIONS 44); the ninth,
the same evening, ruled three — 467, 468, 469 — three clauses in the
commander's chain, after the loop stopped at its own reap
(`docs/audit/2026-09-21-fable-pass-9.md`, DECISIONS 45); the tenth,
2026-09-22, ruled two — 480 and 488 — both guards, each made to fail
first: struck text no longer counts against a section's ceiling, and
`rondata::diff::coverage` pins every key the dump prints that nothing
reads, 239 on twenty paths on its first run
(`docs/audit/2026-09-22-fable-pass-10.md`, DECISIONS 46); the
eleventh, 2026-09-23, ruled nine — 503, 507, 508, 509, 513, 517, 526,
528, 565 — three guards each made to fail first (the lower map, the
closed chapter, the ledger's own list), a lint, a receipt field, five
clauses, and 526 to the ordinary list
(`docs/audit/2026-09-23-fable-pass-11.md`, DECISIONS 47); the
twelfth, 2026-09-23, ruled eight — 574, 575, 583, 584, 596, 598, 605,
612 — a guard made to fail first both ways (a capture booked on a map
cites the window the ledger holds), a parser taught the one-line
literal and made to fail first, a dying floor made an exact pin, the
chapter form's fifth point and chapter seven-b booked as item 628, two
clauses in `CLAUDE.md`, two amendments to `docs/EMULATOR.md` §8 and
`docs/ORACLE.md`, and 598 to item 620's brief: `inside_up`'s container
row goes into `compare` inside the next East Indies widening
(`docs/audit/2026-09-23-fable-pass-12.md`, DECISIONS 48).
The thirteenth, 2026-09-23, ruled nine — 313, 630, 638, 639, 645, 649,
656, 667, 670 — a waiter for a detached capture made to fail first
(`tools/gamelog/waitrun.sh`), the golden-line guard rewritten over every
pinned word and made to fail on the line that fooled it, the gate
reordered with clippy first and a `Gate steps:` line after a test saw
the old order leave clippy unreached, the memcap timeouts, a receipt
that reads its cfg, `Block::fields_of` and fourteen `BUILDDATA` keys
pinned after the pin failed first, two clauses in GOLDEN §3, and 313
closed on three tranches without a forgotten reap; 527 stays with a
commitment and 677 is filed (`docs/audit/2026-09-23-fable-pass-13.md`,
DECISIONS 49). The fourteenth, 2026-09-25, **built 527** — the compared
recorder and its pin, made to fail first both ways, 137 registrations on
the word's window, and its first run caught a comparator comment claiming
a comparison the code does not make (728) — and ruled 687, 692 and 726
(a measurement, a counter, a clause), the golden-lane waiter from the
table (656's sequel, made to fail first 2/2/2 → 0/1/2), the killer clause
(711), the chain's remote branch (727), and a journal's Loop line filed
at its merge; 677, 685 and 697 stay, each with its next step named
(`docs/audit/2026-09-25-fable-pass-14.md`, DECISIONS 50).
341 closed 2026-09-18 by item 363.

(452) and of the same family as (449): an instrument that quietly stops
looking at the moment the thing it measures happens. Both are guard
shapes, not mechanics, which is why they are here and not in the queue.

(281) it read `ps rss`, which over-counted the mapping 280 removed — ~1.3 GB
of 260's 8,816 was run58's clean text — and a 2 s poll under-reports a
sawtooth. Still wants the fixture with teeth: past the cap, dead in N
seconds, exit 137.

(341) closed 2026-09-18 by item 363: `live_session.stage()`'s window and
categories are caller-supplied, the golden record's first run used them, and
`docs/RUNS.md` run101–run105 is the evidence. The lane's real constraint was
never the cursor — it is that a launch from inside Claude Code's own process
tree gets no window at all (`nodrv_CreateWindow`, dead in 3.8 s, 0 frames),
so every launch goes through `viadriver.sh` (`docs/ORACLE.md`, "The
click-free lane needs a window").

## Loop, filed 2026-09-22 — the chain's own defects

The same section, split on 2026-09-22 when it passed the 16 KB ceiling
the guard sets. Everything in the heading above applies: these are the
steering pass's, never a worker's, and a commander never spawns one.

(677) **Birds are owner 9, no dump prints them, and five landings this
tranche touched their rolls.** 648, 652, 657, 661 and 669 each met a
`9/6` draw (`think_bird`, `set_anim+0x104b`) that no record on either
side could be compared against; 652 took a packet to read one cursor;
parked 675 names the same blindness on Great Lakes' word. The fix is
the AI dump's shape (367, run114, DECISIONS 41 §6): a tracer proxy on
`Unit::think_bird` — position, heading, anim per call — so the bird is
a value row and not a draw count. Filed by the thirteenth pass from the
journals; not built. **The fourteenth pass priced it and did not build
it**: the tracer's proxies log a site's *arguments*, so a `think_bird`
proxy logs `this` and the frame for free and the position only with a
new proxy kind that reads memory (`tools/trace/tracer.c`, C, validated
by a capture); 715's new word still spends its third draw on
`Animal::think_bird+0x82`. Stays; the AI lane's next capture stanza may
carry the cheap half.

(685) **`tools/recomp/step4.py` cannot run `do_frame` on a packet from
597's plan**: a stack read falls outside the mapped ranges. A per-function
call on the packet was enough for 680; a question that spans a whole
frame would not be (`docs/journal/2026-09-24-item-680.md`). **The
fourteenth pass named the fix and did not build it**: the plan copies
the process's private data and not the game thread's stack, and
`do_frame` reads its caller's frame on its nineteenth instruction; the
packet's tracer build adds the thread's stack (the TEB `find_teb`
already locates carries `StackBase`/`StackLimit` at +4/+8) to the
ranges, and the next packet's stanza says so. Stays.

(697) **`cover=1` hangs the click-free golden lane before frame 0.**
run185, chapter ten's coverage re-run (item 693), hung with the
`callwin` proxies and the autostart build, and the runner's timeout held
the single lane until 14:17, about an hour. run184 at `cover=0` ran on
the same lane. Until a coverage capture runs there, no issuer chapter
can move the census's *entered* column, only the cited one (692). **The
fourteenth pass named the probe and did not run it**: `docs/ORACLE.md`'s
three six-second probes — `window=0-3` + `cover=1` under
`WINEDEBUG=+seh`, no lobby — on the autostart build alone, then with the
`callwin` proxies, which is what run185 added over the `cover=1` runs
that work on the queue lane. Stays; the cited column moves without it.

(727) **Thirty-seven merged `origin/worktree-att-*` branches stood on
2026-09-25** — every one 0 unmerged. A worker pushes its branch, `ccc rm`
deletes the worktree and the local branch, and nothing in the chain
deleted the remote one; the user saw `att-673` from a phone. The chain
clause is in `CLAUDE.md`; **the sweep is the user's** — the pass's
classifier refused `git push --delete` as destructive — one command:
`git branch -r | grep worktree-att- | sed 's|origin/||' | xargs git push
origin --delete`, from any checkout, after `git fetch --prune`. Closes
when the listing is empty.

(730) **A spec line that names its own contradiction is a cheap catch**
(722's Loop line, filed at its merge). CITIES §2.6.4 said `num_wonders`
counts *finished* wonders beside §3.3's `≤ 1 + Egyptians`, which only
reads right if a site counts itself; the filter stood on that line for
the whole tranche. A reader, or a guard, that flags a count's stated
domain against its own bound would have caught it. 722 also reports
`waitrun.sh` exiting 0 cleanly on the queue lane: 656's sequel held.

## Parked by the fourteenth Fable pass, 2026-09-25 — names no score

(728) **A building's `orig_type`, `damage` and `damage_frac` are compared
by no shared instrument.** The compared pin's first catch, the day it was
built: `harness::compare`'s own comment on the building loop says
`orig_type` "was parsed and neither compared" and is compared now, and
the code compares the position only; the hit-point pair is compared for
units (item 484) and not for buildings. Every capture at any detail
level prints all three. A widening, cheap, and it may move fourteen
pinned widenings; returns when a score names a building's type or hits.

## Parked by the tenth Fable pass, 2026-09-22 — names no score

(476) **f10234's three value rows**: `0/5 order:length` 2/1, `0/5
orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. Booked on the
word's frame when the word was 10234 (`docs/ORDERS.md` §18.3); the word
is 10277 now, the rows are a citizen's flee and a gather field
forty-three blocks under it, and no draw on the word's frame names
them. Returns when a takes-chain to the word does.

(342) **Host choice seats the scholar on the wrong university** — 338's
residue, re-pinned three times, and last `1/55`. Names no frame and no
floor; it stood in the queue through four passes on the strength of
being true. **Re-measure before diagnosing**: the vector `(768, 9984)`
is the claim, never the count. Returns with East Indies (444), whose
word it sits under.

## Parked by item 485, 2026-09-22 — 683's two sides, and two callers

(491) and (492), the two sides of 683 itself, were **promoted to the
queue by the tenth pass** as item 491 — a finding on the headline's
frame books, and the rules slot had stood empty behind a `Golden:` line
that named a landed item.

(490) **`(384, ATTACK3, 16)`'s release node** — the one launch-point
row the chapter-two widening still carries: `damage 1/6` at **680**
against the dump's 679. **Unmeasurable on the disk we have**: the
arrow lands inside its own launch frame, so no `AMMO` block is ever
written for it. Needs a capture with `AMMO` on a slinger volley at
longer range — the first genuine capture question in twenty landings.

(493) **The attrition caller's threshold.** `Sim::attrition_tick`
still kills on the squad-sized `health <= 0` — the same defect item
485 fixed in `Sim::take_damage`, one caller over, on
`Object::take_damage`'s `attrition != 0` arm. `docs/ATTRITION.md`'s,
not COMBAT's. **No capture on disk has an attrition death**, so
nothing scores it today; it parks until one does, or until the
namesake mechanic is worked deliberately.

## Parked by item 491, 2026-09-22 — a rolled arrow's landing

**Promoted to the queue 2026-09-22 as item 495**, by item 510, which
measured the takes-chain this entry was missing. The arrow spends no
draw, but the wound it fails to deliver decides which unit the original's
bowmen target, and that choice is what the rules headline's word is spent
on: 510 forced the facing on 725-727 alone, changed nothing else, and the
word went **725 to 743**. The frame, the rows and the float constraint
are `docs/COMBAT.md` §41.3 and §42.2; the queue carries the booking.

## Parked by item 481, 2026-09-22 — surfaced by a ceiling, nobody's item

(486) **`order 1/4` at 685 and `pos 1/4` at 686, a citizen forty
thousand units from the engagement.** Guy type 50, `myhits 40`, GATHER
on uid 3. Surfaced when 481 lifted `chapter_two_s_word_frame_is_widened_whole`'s
**ceiling** to 687 — the same shape as item 470's floor hiding `0/10`
at 630 — and re-measured with 481's change reverted and the window
left wide: they stand, so they are pre-existing and nobody's.

Same family as `order 0/5` (`docs/COMBAT.md` §32.5). Parks because it
names no headline frame and no floor; promote it if a chapter-two word
ever reaches 685, or if the §32.5 family turns out to be one cause.

## Parked by item 479, 2026-09-21 — wider than the word it came from

(482) **`Unit::think@005f6e40:104-134` has a second `near_o` reader, and
it fires thirty-one frames in thirty-two.** An idle captain takes
`add_attack_order(near_o, QUEUE_NEW)` when it is in range and **skips
the search entirely**; this crate searches on the phase frames and does
nothing on the others. `docs/COMBAT.md` §37.5.

**Not the current word** — `think` is reached from `do_idle` alone (one
caller, grepped) and `0/9` had a MOVE at its head — which is why it
parks rather than queues. But it changes what **every idle captain in
every capture** does, so it is the widest unlanded thing on file and
should be the first row a wave picks up when a captain's idle behaviour
is next in question. Whoever takes it should expect it to move more than
one word, and should measure before and after on both maps rather than
on the frame that sends them.

## Parked by the eighth Fable pass, 2026-09-21 — a guard a worker can take

(375) **A staged run is indistinguishable from an unstaged one, and the
borrow of a checksum trace is ungated.** Found by item 364, 2026-09-18,
while measuring its own first golden word as an artefact.
`borrow_from_siblings` gates `frame_seeds`/`frame_guys` on
`init.checksums.last().seed`, but borrows the **setup checksum trace
itself** ungated (`if init.checksums.is_empty()`), which makes that gate
compare a borrowed word with itself. Harmless where the donors are the
same game (run6/7/9); not harmless for the golden record, whose GAMEINFO
is **byte-identical to run11's** — map 14, seed 12345, size 2, the same 34
option fields — and whose setup word is the same `1003723497`, because the
script's first line runs at frame 0's `do_frame` entry, *after*
`Game::init`. **No gate on the map, the lobby or the setup word can tell a
staged run from an unstaged one; only the caller knows.**

364's runner does the right thing by hand — keeps the borrowed setup trace
after asserting its last word equals the trace's frame-0 entry word
(`0x3bd39ae9`), and refuses the per-frame records. The guard shape is to
make that the only possible behaviour: a golden run declares itself, and
the borrow refuses per-frame records from a run that did not. Written up
because the next golden capture will hit it and a silently-passing
comparison is what it looks like. Not urgent — see (376) for why.

## Parked by the seventh Fable pass, 2026-09-21 — two words without a widening

(444) East Indies' word without a widening was **promoted to the
queue by the eleventh pass** as item 573, the day the lower-map rule
became a guard: the word had been the lower of the two since 2026-09-21.

## Parked by the sixth Fable pass, 2026-09-19 — a guard's first run

(440) **9582, one `use_market+0x1ed` short.** The AI headline's next
parting, measured by item 432, 2026-09-19, under the probe that gives the
Scholar its two per-city values: ours spends **one** `Leader::use_market
+0x1ed` where the original spends **two** — the market's sell rotation,
the same site as 9382's agreeing pair. Everything else on the frame matches
entry for entry (`make_stuff+0x221` ×2, `+0x63d` ×2, three `set_anim`, two
`Ammo::init`, `Farms::inc_time`), and 9581 and 9583 agree whole.

Parks rather than queues because **it is not reachable on the real tree
until `create_units` computes per-city** (438). It is booked by its frame
and its draw delta with no mechanism named, and it replaces 9518's scholar
birth, which was probe 4's artefact through and through.

(429) **A positive i32 wrap is live wherever `create_units`' tail is
reached.** Measured by item 422, 2026-09-19, on frame 9382:
`out = wm(fac, wm(want, val) / divisor) / 256` with `val 42,000,000`,
`fac 256`, `want 20`, `divisor 25` gives `256 × 33,600,000 =
8,601,600,000`, which wraps as i32 to **11,665,408 — positive** — and
divides to exactly 45,568. The original's product stays inside i32
(`5,755,741 × 256 = 1,473,469,696`) and does not wrap. **The sign is the
defect**: §45 documents this same overflow in the arm next door and guards
it with `out < 0 → 9,999,999`, so a wrap that goes negative becomes the
ceiling and is caught, while one that goes positive becomes a plausible
small number that ranks below a Citizen and is not. 422 pins the wrap as
an assertion on its own frame; this parked the general case as a defect.

**Corrected in place, 2026-09-19, by 430's second probe — the wrap is
fidelity and this item was wrong.** Computing the tail in 64 bits so
nothing wraps costs **2,727 frames**: the word falls 9510 → 6783, because
the original is a 32-bit engine whose own `imul` wraps and this crate
tracks it that far *because* it wraps too. So the overflow is not a
defect; it is a wrong input carried faithfully into a wrong answer, and
the lever is entirely upstream in `val`, `want` or `divisor`.
`offer_value`'s doc comment now says so with the number beside it, and
`LONG_WORD_GREAT_LAKES` pinning 9510 means a well-meant "fix" of the
overflow **fails the floor** rather than passing quietly.

What survives is narrower and still true: **`out < 0 → 9,999,999` catches
only half of a wrap's range**, so a positive wrap is invisible where a
negative one is caught. That is an observability property of 32-bit
arithmetic the original shares, not a defect in the expression, and where
it matters is a question nobody has asked yet.

(427) **Twenty-four frames pass before chapter two's first arrow.**
Observed by item 415, 2026-09-19, and **free to measure from run112, which
is already on the disk** — no capture, no screen. Parks because it names no
score: it is a delay this crate and the original may well share, and
nobody has compared them. Worth an hour the day a combat word lands near
it, and worth nothing before that.

(425) **A name a pinned constant carries for an index has nothing to
disagree with.** Named by item 408 while settling 423, 2026-09-19, and it
is the reason that defect survived from run19 to today. Both sides of the
leader diff generate their key from the **same** `GOODS` array —
`rows()` at `leader.rs:87` and `theirs()` beside it both `format!` the name
in — and the dump prints `bucket` as six bare values, so the index is
positional on both sides and **the label never participates in matching**.
The pinned literals in `PARTS_ON_RUN19/107/111` then check only that the
string matches whatever `GOODS` currently says. So `GOODS` and its pins can
be wrong **together, consistently, forever**: the specifications got it
right, the decompile export got it right, and 858 tests never looked,
because a self-consistent label has nothing to disagree with.

The guard is the check nobody wrote, and it is deliberately **wider than
`GOODS`**: every index-to-name mapping the diff layer pins is compared
against the type record, `GOODS` being the first instance. If others exist
it finds them the day it lands; if none do it costs three lines and says
so. A worker's item, not the pass's — a correctness guard about the port
rather than loop machinery. Make it fail on purpose first.

**Its sibling already exists, which is the argument for building this
one.** Item 422 wrote `assert!(45_568 < 234_782 && 234_782 < 5_755_741)`
— three literals agreeing with each other, unable to fail — one commit
after landing 423, and clippy's `assertions_on_constants` killed it in
ninety seconds because the gate runs `-D warnings`. That class of
decorative check has a working guard. **The label class has none**: no
lint knows that `GOODS[2]` should read "wealth", which is why one sat
wrong from run19 until someone traced a number to the array that named
it. Nothing to build for the assertion case; this item is the gap.

**Measured 2026-09-19 by 423's own fail-on-purpose**, which is what tells
this apart from a load-bearing label: with one row left on the old name,
every value comparison passed — 33,536 field-frames — while the `parting`
equality failed in **exactly one position**, `income[2:wealth]` generated
against `income[2:metal]` pinned. The pin guards the literal against the
generated name; nothing guards the generated name against the truth.

**The same shape one level up, and 423 found it in its own writing**: its
`docs/AI.md` §48.4 said "three pinned constants across three items", an
estimate written into a specification without counting, and the count is
one pinned constant and one assertion key. A number asserted in prose has
nothing to disagree with it either — struck in place and pointed at the
measurement. Whether that generalises to a guard over counted claims in
`docs/` is the pass's to judge; it is named here so it is not lost.

(423) closed 2026-09-19 by its own landing (`docs/journal/2026-09-19-item-423.md`);
the entry stood live a week after, and the eighth pass's number guard is
what found it. **`GOODS` is mislabelled in `diff/leader.rs`.** Found by item 414,
2026-09-19, while tracing what pays for a Scholar. Index 2 is named "metal"
and index 2 is what pays this Scholar's 40 **wealth**:
`sim::economy::Resource` has Wealth 2, Knowledge 3, Metal 4. **No verdict on
any capture is affected** — the comparisons are index against index, so
every number that has ever been checked was checked against its own
counterpart — but the *names* pinned at 2, 3 and 4 in `PARTS_ON_RUN19`,
`PARTS_ON_RUN107` and `PARTS_ON_RUN111` are the wrong resources, and a
reader who trusts them will reason about the wrong good. **Corrected in
place 2026-09-19, before the item ran**: the authority is the type record,
`rise.pdb`'s own `TypeIndex` — `0 FOOD 1 TIMBER 2 WEALTH 3 KNOWLEDGE
4 METAL 5 OIL` — which is `sim::economy::Resource` exactly; and the blast
radius is **six strings in one file, every one at index 2**, not three
constants across three items. There is no wrong row at 3 or 4, because
who=1's knowledge and metal never parted on those windows, and the
seventeen rows at 0, 1 and 5 are already right because those indices agree
between the two orderings. **No document changes**: `docs/AI.md:227` and
`docs/ECONOMY.md:861` already say wealth. Parks because it names no score;
a worker's to take, not the pass's.

(418) **Order coverage is a separate axis, and its unit is a native
issuer.** Found by item 365, 2026-09-19, while designing the golden
record's chapters. The cheat channel **stages state**; it does not order.
So seven of the eight new chapters add no order class the tree already
enters, chapter six adds one, and `bird` — console table case 82,
`Unit::add_air_patrol_order@005e4350` — is the only console command that
issues an order at all. "Every order class" is therefore not something the
chapters can deliver, and `docs/GOLDEN.md` §13 is the order-class-to-issuer
table DECISIONS 41 §5 was waiting for: coverage is counted in native
issuers reached, not chapters run. Parks because it names no score yet; it
returns when one names it.

(412) **Fourteen document-address pairs cite a dead-listed function.**
`docs_guard::a_dead_listed_address_is_cited_only_where_pinned` reads
`docs/EMULATOR.md` §4's fifteen and pins the citations standing on the day
in `DEAD_CITED`: `LeaderData::is_human@006ec170` in ARMY, COMBAT, GROUPS
and ORDERS; `find_wpath_army@00683730` in GROUPS and ORDERS;
`get_estimate@00688310` in PATHFINDER and RUNS; `locked_transport@006d5230`
and `find_dock@0065cfd0` in TRANSPORT; `set_domain@00633390` in COMBAT;
`add_attack_order@00622ce0` in ORDERS; `set_gathered_at@006b46b0` in AI;
`cos_table@00a469f0` in MOVEMENT. Each cites a standalone body the linker
kept while the live copy is inlined in a caller, so the claim rests on
whatever else backs it — run58's dump for `is_human`, a listing for the
rest — and the reconciliation is to say so at the citation and delete the
row. Names no score; a worker takes one document at a time, and the row's
deletion is the landing.

(413) **Seventy-seven constants the specifications name that no crate
carries.** `docs_guard::a_constant_a_document_names_is_built_or_pinned`
pins them by file in `UNBUILT` — AI 20, ORDERS 9, TECH 7, CITIES 6, GOODY 6,
ARMY 5, ECONOMY 5, PRODUCTION 4, GROUPS 3, and one or two in nine more —
and a file may only shrink. Three kinds, to be told apart one file at a
time: a field offset written without its `+` (spell it `+0x..` and the
guard stops reading it); a value named for context the code never needs
(say so beside it); and a constant read and never built, which is the
kind that cost a month at `0x66`. The third kind is an item on whichever
track its frame is on; the other two are a document edit and a lowered
pin. Names no score until a row does.

## Parked by the third Fable pass, 2026-09-17 — names no score

(310) **Nothing decrements the muster on death** — not `by_type`,
`by_group`, `control` or `active`. The original's `Unit::close@0060ee50:235`
undoes all three and this crate has no counterpart. No unit of players 0
or 1 dies in either window, so no diff reaches it, which is why 303 could
land `Sim::track_unit_type` correct in both directions of `set_type` and
still leave this open. Falsified by a window containing a death. AI §37.

(305) **The make list's five building values, and a factor of ten** — the
building producer's rows `MAKE[0]`..`MAKE[4]` still part, with `MAKE[2].cat`
beside them, and two are exactly ten times out: `MAKE[1]` 202500 against
2025000, `MAKE[2]` 165000 against 1012500. A different producer from 302's
`upgrade_units`, and the factor-of-ten shape is as strong an oracle as 302's
factor of two was. Value diff on disk, no capture needed. AI §36. Parked
because 302 and 303 closed make-list rows and moved no word.

(278) **`Unit::work@0060d180:440` is a second `set_new_location(…, 1, 1)`**
and nothing models it — two units of one type within `0x180` are pushed
apart by half their separation and **both snapped**, gated on
`field_0x82 < 0` and the order's `+0x30` vcall (read, never run). Ruled out
for 271's 6937; `Guy::last_pos` makes the signature searchable on every
dump: a unit that moved whose figure has `last_x == x`. run79's squad first.

(220) **TECH §13's twenty range blocks.** (209) **the once-per-game events a
dump install swallows** — a `set_*` whose **return value** drives an
irreversible record; **takes 224**. (224) **`mil_trainers`' other three
writers** (AI §29.4) — a trainer that changes city, upgrades in place or is
captured is filed by neither; referenced since 09-04, booked 09-17. (203)
`mark_behind_tiles`' `0x4` is a building *finishing*; (240) the merchant's
`gather_down`/`special` −1 at 6929, **takes 184**; (216) `create_buildings`
offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945. (245)
`focus.sh` matched a concurrent worker's shell on run84.

(255) **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
never run** — every joiner on disk is farther than `0x480` from its army's
`get_unit(0)`, so the falsifier is one born beside its army. (256)
`come_out`'s three `action_move_to` sites stay unreached; (254) its ring
has one sample (7284) and wants a second disembark. (167) run61's two (SYNC
§3.9). All four came off 253, which 271 closed.

(247) **An upgrade is an in-place guy-type change on the standing unit** —
run76's **6737**, three Archers going guy **170 → 177** keeping `(who, o)`
and `group 64`; East Indies' `1/32` does 340 → 341, the danger row moving
by `(110 − 100) / 2`. (181) CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4, §4.1): **18** fields the harness
names nowhere, **46** one capture names — item 308 moved both on 2026-09-17,
`start_dist` off the uncompared list and `uid` onto the single-capture one; blind spot `avg_speed` (210) — it
counts *fields* and cannot see a record the parser never visits, which is
what hid 252's dumps. Uncounted: (88) the blind list — **802 cited, 650
entered, 152 never** since coverage came back 09-08 (ORACLE), not the 101
of 617 this line said for a week; (72)
every `+0xNN` a document pins vs its module; (89) the guard; (35) VISION §7.

(269) **The third axis: a field compared at the wrong width** (DATALAYER
§4.2, from 265). Ten `CityData` counters are `uchar` in the type record and
`i32` in `ai::CityAi` — `busy` and `gatherers` have bare byte writers of
their own — and `pop` is an eleventh on `sim::City`. None has been seen to
wrap; the falsifier is a capture where the sweep's count and the producers'
decrements cross zero. Same question one record up, for every
`char`/`short` of `LeaderData` held as an `i32`. A guard wants the PDB's
widths beside the sim's structs, not a grep.

(270) **The fourth axis: a word compared one bit at a time** (DATALAYER
§4.3, from 267). The ledger scans the differ for a field's *name*, so
`unit_masks` has been on neither list since the packed bit got a row while
one of at least eight modelled bits was actually compared — and the missing
one, `0x100000`, named Great Lakes' run-up cause on its first run. Nine
dumped masks want a per-**bit** census: `unit_masks`, `unit_masks2`,
`guy_flags`, `node_flags`, `city_flags`, `leader_flags`, `leader_flags2`,
`build_flags`, `role`. Same tool as 269.

(257) **Nineteen kept tests compare a torn block** (252): a closing dump is
frame n but for the one unit the quit caught mid-update, and every nested
archive's last `FRAME n` body *is* its closing dump. No live case found,
unaudited; fix is `compare_shutdown`'s n−1 allowance.

(175) **The uber chain past its birth**: `Objects::init_unit` threads
`uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
COLLISION §3/§7, (73) `UnitData::group`'s back-pointer, (56) ARMY §13.

(161) **The make-list block is 2,500 frames behind East Indies' word**:
`create_buildings` first runs on 9982 (AI §25), so `building_value`,
`gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

(211) **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
doubling; the group cap, gated on `action_type == 0` (219) — it wants a
grouped unit holding no action. (229) a figure in melee does not step its
clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM §5), and that
arm waits on a melee frame. **312 names this first** if the 7679 figure is
in melee.

(291) **The AI's caravans are not linked** (item 661: the `trade_val` and
`vans` halves were the harness comparing against a literal 0, Loop 670) —
`vans.length` 0 against 1 and
`trade_val` 0 against 128 on **both** of player 1's cities, 246 of 246
blocks (287). That names two of 285's eighteen fields and gives them a
mechanism, and wealth is what a market buy spends, so it is a live
candidate for 290's shortfall. (288) the gull's `do_strafe`, unmodelled.
(274) run87's `1/26`. (268) `AnimalData::ox`/`whom`/`aid` carry nothing —
−1 on all forty animals on all 247 blocks. Closed as answered, not open.

(285) **The CITY record parts on every block and nothing asserts it** — 18
fields on 246 of 246 of run89's window, found by widening the whole record,
and **no window test on either map asserts `city_diverged`**. Two of the
eighteen are named now (291, the unlinked caravans). `1/3`'s `(+192, +192)`
`MOVEORDER` was booked here and closed itself when 284 landed the
air-physics fix.

(273) **`refresh_group_order` re-origins on the order's `form_id`, not the
member's list position** — `713ac3` reads `[eax+0x10]` off the `GroupOrder`
the `+0x94` vcall returns, where `Sim::group_refresh_order` uses
`g.list.iter().position(member)`. `do_group_move` step 2 rewrites `form_id`
every frame, so the two agree except where membership changed and the
follower arm has not run: wants a `GROUPS=1` window across a death or a
join in a marching formation. (275) `MoveOrder::facing` still does not
score; 267 split the two mechanisms, so re-read that. **304 is the nearest
live window** to this — a formation ending early on run76.

(315) **run90's `1/7` names no blocker on 7820** — `collide_o` and
`collide_who` read −1 where the original has 6 and 1, with `collide`
itself, the count, agreeing. New on 2026-09-17 with the suspend wiring
(items 301/304), and the window cannot price it: every position, every
angle, every order record and every draw count over those 111 blocks
agrees, and the two Merchant constants are the only parted units left.
Pinned in `run90_s_window_is_east_indies_shuffle` so it cannot move in
silence. Parked because it names no score — the falsifier is a capture
where a missing blocker identity changes a decision.

(316) **Sixty-two other callers of `clear_partial_path` are unchecked**
— item 304 fixed `kill_current_path`'s and no capture reaches the rest.
The `Group::action_*` and `think_carry*` families have **no counterpart
call in this crate at all**, which is the shape 304 turned out to be, one
level up. Parked because it names no score; it comes back the day a
window contains one of them.

(325) **`MAKE[*].city` is ours + 1 on every offer** — this crate's city
array puts the human's at index 0 and the AI's at 1 and 2 where the dump
reads 0 and 1. No offer in run19's window is chosen by the index, so
nothing scores it; the falsifier is an offer whose choice depends on the
city. Item 323, AI §38.

(326) **Three tech `val`s part from before run19's window** — Empire
2,100,000 against 1,800,000, Mercenaries 165,000 against 216,000,
Mathematics 82,500 against 63,000, all `research_techs`' arithmetic. A
value diff on disk, no capture needed; parked because the window that
prints them is not a scoring one. Item 323.

(333) **`1/28`'s path stack is 41 where run19 says 42** — an off-by-one
already present at 8186 and belonging to `find_wpath`'s plan near the
goal, found by item 329 and deliberately not closed by it. The chase and
the delay are right either side of it, so nothing on the word depends on
it; the falsifier is a plan whose last leg the count decides. PATHFINDER
§21.

## Parked from the lab, 2026-09-17

(309) **`find_upath`'s pre-walk give-up exit targets the wrong label** —
`00683082` is push-and-return-length where this crate's `break` falls
through to the near test and the search. Unreachable today for a
transport-capable unit and no capture reaches it, so 301 recorded it in
PATHFINDER §18.4 rather than changing it. Parked because its falsifier does
not exist on disk.

(311) **The zero-pop predicate disagrees on paper and nowhere else** — the
original counts a `control_cost == 0` unit only when `is(0x134)` or
`is_gov_hero`; this crate's muster seams apply no test at all and its sweep
skips every zero-pop unit outright. Two different wrong answers that agree
on every capture, because nothing on disk separates them. AI §37. Parked
with 309: a reading-only disagreement with no falsifier.

(298) **The click-free capture lane starts 6 of 10 pairs** — a
`RON_AUTOSTART` tracer build replaces the two menu functions and clicks
Start from inside the modal loop; every success matched 19 logged bodies and
1,401 frame seeds against a hand-driven run. The failures are 180 s timeouts
and exit 40 before the menu; two faults map to the WoW64 transition RVA
0x1139, and moving the hooks off bulk restores (226's family) did not cure
it. The factor-isolation experiment is paused
(`docs/lab/2026-09-09-paused-runtime-experiment.md`). Falsifier: a ten-pair
cohort at 10/10. Lab rows L18, L19, L28, L29.

(299) **A single Gaia reseat correction hides a 556-frame heading
difference** — run69, Great Lakes: of seven actual reseat writes across
three intervals, removing tick 99's leaves this crate's Gaia heading off the
original's for 556 frames, only the control matches at 106, and all seven
converge by 3001 — invisible to the player comparator throughout. Lab rows
L53, L54, `docs/lab/2026-09-10-single-reseat-interventions.md`. Wants a
Gaia heading row in the differ; unscored today.

(300) **The original's command API is unused, and the replay adapter lowers
only `MoveTo`** — `CommandManager` has 68 `issue_*` entries;
`CommandManager::issue_move_to@00941720` builds the command and appends it
to `local_package` after `CommandManager::check_accept_issue@00940a70`,
while `tools/fuzz/scenario.py` teleports. On this side `input::Stream::one`
applies selection and `MoveTo` and skips production, repair and market. Lab
row L08: acceptance is not emission — package capacity can refuse after
selection changes — so an adapter needs acceptance, emission and
processed-frame witnesses. A prerequisite for manufactured falsifiers and
phase 5, not headline work.

## Parked from the queue, 2026-09-07

(277) **An age gained through a cascade is unmodelled** — `Sim::gain_tech`
reads the age gate off the type the call was made with, as the original does,
but the original **recurses into `gain_tech`** per cascaded grant where this
crate flattens the cascade into events, so a cascaded age would take 271's
snap arm there and not here. Parked to seat 290/291 under the cap, on the
ground that **no capture on this disk gains an age other than directly**, so
nothing can falsify it today. It is cheap to unpark: `BuildDump::max_age` is
parsed, so the falsifier is a grep rather than a reading — a block whose
`max_age` moves on research that is not one of the seven ages. Cross-item
constraint, and the reason this is not a silent drop: **271's snap arm is
live code**, so anyone touching `Sim::gain_tech` inherits this question.

(248) **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants a
retelling pass before anything is added. Parked to seat 283; its remaining
terms are **closed** — handicap, temple and fort are unreachable in any game
that runs here — so nothing is waiting on it and the retelling is owed only
to whoever next adds to that section. Read it before re-booking any of the
three. (AI §2.1's `check_explore`.) No cross-item constraint: the size ceiling
is per-section and `docs_guard` enforces it, so a worker cannot trip over this
without being told by the guard itself.

(234) **Four rules of the turn/idle animation neither side has** (ANIM §9),
written and never landed — branch `rescue-234`. Parked by the commander to
seat 276–279 under the item cap, on the item's own words: the limbs move no
score, since §4.7's turning types are in neither scored game. **The two
static assertions are the cheap half** and are what to take first if it
comes back. It carries the ledger's nine `unverified` rows on the way, so
whoever unparks it inherits that audit. Nothing here is a cross-item
constraint: no live worker is in `crates/sim`'s animation code.

## Steering candidates, booked 2026-09-07 with Ramon

Neither of the two below is a mechanic and neither moves a word, which is why both park
rather than take a slot under a cap that stood at 18 of 18 the day they
were booked. Ruled by the second Fable pass the same day
(`docs/audit/2026-09-07-fable-pass-2.md`): 293 closed there, 292 stays,
and 283 came here from the queue to sit behind it.

(292) **Nothing in this tree has ever been profiled** — no bench target, no
criterion, no flamegraph, no samply, and no mention of Instruments anywhere
under `docs/`, `crates/` or `tools/`, grepped 2026-09-07. Every measurement
the gate has ever been given is **memory**: the 15,128 MiB peak at two
threads, the memcap in front of it, and the ratchet items 235, 260, 280 and
283 that came off it. Its **256 seconds** is a wall-clock fact with nothing
under it — no split between parsing a dump and comparing it, which is the
obvious first cut when run89 is 121 MB and run90 72 MB of text. That
ignorance is upstream of more than the gate's runtime: 283 proposes a
`#[global_allocator]` on reasoning about what macOS's allocator keeps, and a
profile is what would settle it rather than argue it. Cheap to start —
one bench target over the differ's parse and one over a sim tick — and it
earns its dependency the moment it prints a number nobody predicted.
**Ruled 2026-09-07**: stays parked with this first cut; the gate's five
minutes is not what the loop was losing time to (293), and the item is
taken by whichever worker next touches the gate's runtime, or with 283.

(283) **The ratchet wants a `#[global_allocator]`, not a mapping** — 280's
+1,563 MiB is *not* live data: a mapping is `munmap`ed at drop, while a
freed `String` of a capture's size is kept by macOS's allocator and cannot
serve the next capture's different size (260's own 5,332-MiB-with-nothing-
alive probe, again). An allocator that returns large blocks recovers most
of it with **no `unsafe` in this tree** — the crate carries it — and closes
235's ratchet for every large owned buffer, not the two that were mapped.
**Parked by the pass behind 292**: the retained memory is not live, not a
score, and not a hazard behind `memcap.sh`; a dependency is earned by a
measurement, and 292's bench is that measurement.

## Parked at the 289/290 merge, 2026-09-07

(296) **Great Lakes' endpoint is eight units short and nobody knows which**
— 290's `get_mod_resource_cap` fix halves the AI's commerce cap on this
lobby's Easiest, which is the right-hand side of every `income < cap` gate
in `create_buildings`, `create_units` and `research_techs`. It took 33
spurious units off the four endpoint rows and on Great Lakes went eight
past the mark: `(1,73)`-`(1,80)`, the last eight object numbers the AI ever
reaches. run53's endpoint is MISC-only, so their positions are known and
their types are not. **The cheapest falsifier is already on disk and was
not spent**: run80 is Great Lakes `LEADERS=9` over [23960, 24000), and that
record carries `num_units` and `num_queued`, which would name exactly which
types the AI is short of. Parked rather than booked because the queue stood
at 18 of 18 and this sits 16,400 frames past a word neither map moved, on
streams that are nobody's. What keeps the change that caused it is that its
own evidence is local and strong — twelve lines of decompile at 006d65b0,
and `rate[0]`, `rate[1]` and `best_good` going from wrong on all 80 blocks
of run84 to right on all 80. Expected in direction, unchased in size, and
290 declined to call it clean rather than papering it over.

**Two senses of one word, kept apart on purpose.** At the endpoint,
`unlinked` is a `(who, o)` the dump has and the simulation does not — a
unit the original built and we did not. In item 291 it is a caravan not
linked to a city. Same word, different counter, and blurring them would
make either number unreadable.

## Parked 2026-09-18, the two-lane session

(372) **The held-out map's 1 is comparable in kind, not in provenance.**
run106 ran on the click-free lane's lobby, not run33/run39's `-config
check.ini`, and its window opens at frame 1 rather than matching the score
runs' shape. Strict comparability wants a run33-shaped capture on map 9 —
a second capture and a second decision, which is why 363 left it. Park,
not bury: the 1 is the generalisation number and the first pass that
reads it should know exactly how it was taken.

(373) **The click-free lane's give-up truncates rather than stops.**
`--timeout` defaults to 180 s and the first held-out attempt returned 593
of 1,900 blocks in a **62 MB file that reads as completely ordinary** —
only `receipt.json` said `success: false`. The silent truncation is the
finding, not the timeout: every check that greps a dump would have passed
on it. In `docs/ORACLE.md` beside the new knobs. A guard shape: a capture
asserts its own block count against the window it asked for.
**Half-closed 2026-09-18 by item 369**: run107's stanza is the first in
`tools/gamelog/captures.txt` to carry that assertion (30 blocks,
9170..9199, no gap). Making it every capture's is still open.

(376) **The sibling borrow is measured harmless on every scored path, and
this is the record of it** (item 364, 2026-09-18, the negative the
commander asked for before pinning). Five Great Lakes captures — run33,
run53, run89, run97, run100 — take 14 `frame_seeds` and 14 `frame_guys`
from run12/run13 at frames 0–3 and 94–103. East Indies takes **nothing**:
setup word `793793043` never matches the donors' `1003723497`. run53 walked
all 24,000 frames both ways gives **word parts at 9362, sequence at 9182,
identical with the borrow and without**; run33's floor is unmoved between
the two columns. The installs are no-ops — our sim already produces the
original's word at each of those fourteen frames, all at frame ≤ 103 and
9,079 below the word. Structurally, `Built::tick` pushes the frame's
labels into `frame_sites` **before** installing anything, so an installed
frame's draw comparison is still the sim's own work; the install can only
re-anchor the next frame. **No pinned floor or word rests on installed
sibling frame data.**

(378) **Put the human back into `census_wars` and `census_strategy` too,
and the residue gets worse before it gets better.** 368 fixed
`census_territory`'s gate (the human is a leader: `plan_strategy@006b9620:159`
tests `leader_flags & 2`, `i != who`, `i >= 0` and nothing else, and on a
one-human-one-AI game the human is the only other leader there is);
`1/other_team_terr 0 theirs 266` is `0/my_team_terr 266`. The same defect
sits in the other two loops, but fixing them makes all five of
`weight_total`'s census facts agree and takes run19's leader residue from
**90 fields to 147** — the make list then parts on `t`, `cat`, `city` and
`val` across nearly every slot, because `active_wars != 0` reaches the
danger word and the region-strategy words as well.

The number is in the row on purpose: a successor that makes a residue
worse before better does not get taken unless its cost is visible.
Carried with it: the original runs a **full census for its human leader**
(`0/gatherers 5`, `0/peasants 5` in every dump) and this crate runs none,
so `census_strategy`'s `weaker` test reads the opponent's `attack` as
nought even once the gate is fixed. That is the part to build first.

(380) **A bare coordinate on the staged channel is a TILE**, `n × 0xc0 +
0x60` — measured by item 364, not read: `add hoplite who=0 4,40` put the
squad head at `(888, 7800)`. `docs/ORACLE.md`'s stored `arg × 768 + half a
footprint` is corrected for the staged channel, and 363's open "the
coordinate argument did not calibrate the way the runbook says" closes.

(383) **run107 opened two families never compared on Great Lakes.**
Player 1's ten `SITE` slots part as a **re-ordering of the same ten
sites** — our `SITE[3]` is their `SITE[1]` and so on; since 688 the values
agree and only the slot order stands, with one low-value site in three
spans (AI §67.5) — and `tech_frame`/`tech_cat_frame[0..3]` sit at
nought against 8382/4976/8382/8182/6376. Both are inside
`run107_s_window_is_the_leader_record_at_the_word`'s 109 fields, so they
are floored and cannot regress silently; neither is on the word's frame,
which is why they park rather than queue. The re-ordering is the more
interesting: the same ten sites in a different order is a comparator or an
insertion order, not a missing mechanic.

(387) **Target selection should walk the world cell's object chain, and
ours is units-only.** Found by item 384 while closing 617's range half.
The crate already maintains the chain; switching the scan to it would
**drop buildings from target selection**, so it is a bigger item than it
looks and it would not have moved this frame. Parks on that second clause
— it names no score today — but it is the shape of a whole family, and
whoever takes it should expect the building half to come with it.

(388) **The golden record's `1/7` and `1/8` get their target from a group
path the sim does not have at all.** 384's other parting note: `near_o`
shows only a squad's captain searches (`1/7`, `1/8` and all of who=0 are
`-1` for 900 frames) and `Group::target_opportunity` spreads the captain's
find, which this crate does not model. Documented in `docs/GROUPS.md` §13.
Not on 617's frame — 386's tie-break is — so it parks until a word names it.

(393) **who=0's members take the captain's order a frame after it** —
`docs/GROUPS.md` §13's mirror of the thing item 391 closed on the other
side. 391 established that `Unit::target_opportunity@005fffc0`'s opening
`while` walks `o_up` to the victim's **captain**, and that the army rather
than the group moves a struck unit (`Armies::emergency` → `Army::process`
→ `Group::action_siege_attack_to`; `Group::target_opportunity`'s branch is
guarded by `type +0x2c8 & 0x10000 == 0` and never runs here). Dump-backed:
at the end of 616 the ATTACKORDER is on `0/6` alone, `0/7`/`0/8` a frame
later. Parks because 392's ring is what stands on the word's frame; this
is one frame below it and will likely fall out of the same model.

(400) **`0/8` plans its chase at 619 to the target's own point** where
the dump has (1224,8280) at 618 — item 395's second successor, dump
coordinates in `docs/COMBAT.md` §20.4. Below the golden word (624) rather
than on it, so it parks; likely to fall out of 399's `Army::process`
model, and worth re-measuring before it is taken.

(401) **Two reading-only residues from 395**, `docs/COMBAT.md` §20.3 and
§20.5: `find_melee_target@005ff9c0` carries its **own copy** of the
non-captain mirror, unimplemented here, whose only reachable caller is
`docs/GROUPS.md` §10's per-member call; and `think`'s shared epilogue at
`5f761a` clears the "could not reach" bit, which this crate never clears
at all. Neither is diff-backed — a run has reached neither — so they are
exactly the kind of claim the coverage section must list as reading-only,
and the kind a blind second reading is briefed with.

(403) **The launch table has one piece missing, and one capture closes
it.** Item 396 measured every release event `unit_graphics.xml` gives the
Longbowman — six rows reproducing all nine launch points to the unit
across three units and two facings — but run17's three Slinger families
are **measured and unusable**, because run17's `GUY` detail is 1 and
nothing there names the animation. A **`GUYS=4` re-run of any Slinger
fight** closes it, and the same shape closes every missile type in the
game. Cheap, and it generalises further than the item that found it.

(407) **A pool dump needs its INHERITED category set high enough, and
"DEATHS off" is the wrong rule.** Item 399 lost a `GROUPS` pool that came
back empty with `DEATHS` already off. `GuyData::log_data@005de6c0` sets
type `0x14` and then walks `set_detail(1,2,3,4)`, and **`set_detail` runs
whether or not the line is accepted**, so the pool inherits GUYS at detail
4 and `check_accept` drops it at `GUYS=2`. The rule is that the inherited
category must be set **at or above that dumper's highest `set_detail`**;
run31 and run92 satisfied it by accident with `GUYS=9`. In `docs/RUNS.md`
run110. Riding along: `tools/explore/golden_capture.sh` cds to
`tools/explore`, so `--cmd-file` must be absolute.

(409) **The ammo pool's order is not this crate's, and a two-landing frame
would spend the draws in the other order.** Found by item 402, 2026-09-19,
while widening the `AMMO` record. `Objects::add_ammo@00658b10` scans its
pool from slot 0 and takes the **lowest free slot**, so the original's
order is slot order with reuse — measured 0, 1, 2, 3, 4, 0, 1, 2, 3 over
run109's nine arrows. `Sim::projectiles` is a `Vec` that
`process_projectiles` `swap_remove`s from, so from the first landing it is
neither: at 9453 it holds a4, a3, a2 where the original's pool reads a2,
a3, a4. **Nothing in run109's window turns on it** — no two arrows land on
the same frame after 9451, and landing is what draws — which is why it
parks rather than queues. It becomes an item the moment a frame lands two.
A slot pool is the fix and it could move the word **in either direction**.
`docs/COMBAT.md` §24.4 names the rule and the consequence.

(410) **Three `AMMO` branches with no capture behind them.** Also 402. All
183 blocks print `traj 1`, so the parser's nested-`SplineData` skip — the
one written by indentation because the log writes no `END` — rests on a
hand-built fixture; **a catapult, or anything whose `ammo_path` is set**,
would make it evidence. Alongside it: `flags` bit 1, `rolling` and
`start_roll_angle` have no behaviour behind them, and nine allocations
from a cold pool cannot say whether `graph_index` wraps. Every row here is
one Longbowman shooting one farm on one trajectory, and each of the three
wants a different shooter rather than a different frame.

## Measured residues, none near a word

(454) **Five residue families never compared on Great Lakes, measured by
448's widening.** 2026-09-21, from 837 blocks and 1,975,563 record rows:
`form` **48 rows** (ours −1, theirs 9 on every unit outside a group — the
largest and the cheapest to read), `dest_angle` 22, `orders_x`/`orders_y`
36 (18–24 units off, one pair 984), `g.angle[0]` 10 (ours 1,431,655,765,
a third of a turn, against 0), `stance` 6 (item 190's, unchanged). All
are **older than the word**: of 298 keys parting over the window, 161
stood on the window's first block and all 49 that opened below the word
belong to a family already standing. So none of them names a score, and
a successor takes one only when a word's frame reaches it.

(455) **Two dumped records that nothing in `rondata::diff` compares at
all.** Item 448, 2026-09-21. `LEADERDATA`'s `score` and `leader_flags` —
four blocks a frame, on **every** capture ever taken — and the per-frame
`WORLD` census (`forest_size`, `mountain_size`, `rock_size`,
`total_metal`, `total_oil`, `goodies`, `land_resources`,
`sea_resources`), which the parser does not even read. The rule this
fails is `CLAUDE.md`'s own: when the original dumps a record, diff the
whole record. Nine tenths of a dumped record once went uncompared for a
month and the first widening failed on its first run; these are the next
two.

(451) **The original runs its war census for the HUMAN leader, and this
crate leaves it at zero forever.** Opened by run115's window (item 390,
2026-09-21) and not that item's: `0/wars`, `0/active_wars` and
`0/active_wars_with` agree at nought for 121 blocks and part on **8001**,
where the original writes the human `wars 1`, `active_wars 1`,
`active_wars_with 2` beside a single `production_step` tick that falls
back to 0 on 8002 — 56 blocks after first contact. **First direct
evidence on this disk** that the census runs for a human at all, which is
what `docs/AI.md` §45 argued from the decompile
(`plan_strategy@006b9620:1511,1557` gates on `leader_flags & 2`, `i !=
who` and the met bit, and on nothing about humans) and what §43's `human`
skip contradicts. Parks because it names no score *yet*: the AI reads its
own `active_wars`, not the human's, so what this would move is the human
leader's own production, and whether that lands on any word's frame is
unmeasured. The three fields are already pinned in `PARTS_ON_RUN115`, so
a successor has standing rows to move rather than a hypothesis.

(371) **`market_speculation`'s two passes fire nowhere below the word.**
Read whole by item 362, both arms' predicates recorded in
`docs/journal/2026-09-18-item-362.md`. It was read because 362 was booked
as the market's and was not — `use_market` implements the decompile line
for line, and `MakeObject.num` was the cause. Parks because neither arm
fires below either map's word: it names no score, and the reading is
banked rather than lost.

(246) step 6's repath rests on run83's single event (239); (169)
`compute_site_stats`, 7,122 of run63's 27,000 site fields; (172) the
`bucket` pair on 5002/5061; (158) the human-leader sweep (AI §23.1); (159)
the `CITY` record's two seams; (146) `train_time`'s nine national arms;
(142) `World::tregion` (PATHFINDER §15–16); (122) 16 of 61 draws without a
`self.mark(`; (153) `TRIBE`; (20) the `Census` rows; (23) the formation
byte's sign, Echelon half (GROUPS §6.4), run52 the blocker; (103) a
woodcutter's clock, 445 v 480 (ORDERS §6.4); (105/45) Gaia's positions,
first bad 1658 (SYNC §4.2); (124) the loop flag is per animation file;
(116) the one `SITE` slot (AI §18); (166) `resource_cap` on five goods
(ECONOMY); (107) `epoch[0]`.

## Older backlog

(39) a 2D viewer over `Sim`; (41) `scenario.py`; a `find_target` block;
run7's orders; a mounted attacker; `calc_gather` non-flat;
`Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.
(306) **The `city` column is one high on every make row that carries one** —
ours 1/2 against theirs 0/1, on all 86 blocks of **both** windows, so an
off-by-one in whatever city index `make_me` is passed. Unrelated to any
value, which is what makes it separable from 305. AI §36.

(307) **`age_p`'s zero-age arm is unexercised** — no type in the shipped tree
reaches "a predecessor of age 0 leaves the walk looking" on either
window, so that half of 302's fix rests on the PE listing and not on a
diff. It is a reading-only claim in a document otherwise diff-backed,
which is exactly what the coverage sections exist to flag. AI §36.4,
§36.6. Falsified by a capture where a zero-age predecessor exists.

(459) **The building half of `ObjectData::visible` is read and not built** —
`Build::do_attack@006228f0:147` sets `visible |= 1 << target_who` after
`fire_ammo` and clears it on the same 32-frame slot a unit uses, and
`Build::do_missile_launch@00622670:70` writes `0xff`. So a tower that
shoots you becomes visible to you and a silo that fires becomes visible
to everyone, and `Wall::update_local_seen`'s third mask term is still a
stub here. VISION §9.1, §9.8. Names no score: no capture on disk has a
building shooting through fog. *Would settle it:* `BUILDS=7 UNITS=3`
over the frames a tower fires, and the `BUILDDATA` `visible` byte.

(460) closed 2026-09-21 by item 462: **falsified by its own
falsifier**, run exactly as written, and not one row moved — no
unit-target path in this crate reads `Profile::x_size` at all
(`docs/VISION.md` §9.8). The 622 destinations were
`find_nearby_target` walking the unit index where COMBAT §12.2
and §18.1 say it walks the cell's own `down` chain, plus the unit
half of `find_attack_pos` (COMBAT §32.2). The candidate was
cheap, precise and wrong, which is what a written falsifier is
for: it cost one run rather than an item.

