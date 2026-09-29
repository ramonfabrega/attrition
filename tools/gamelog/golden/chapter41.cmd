# Golden record, chapter forty-one — CENSUS row 7's last seven: a
# computer's Airbase sends its Biplane over its own City under attack, and
# a trireme's hits on a Barracks spill onto the Citizen repairing it.
#
# docs/GOLDEN.md §50 (item 1182; docs/DECISIONS.md 41 §1 and §5, 49).
# A cast of its own on the golden start, as chapters thirty-eight to forty.
#
#     0 `!ai off`                                   every chapter's
#   606 `add airbase who=1 212,104`                 H 1/2006, (40800, 20064)
#   608 `add biplane who=1 212,118`                 P 1/6, outside H
#   610 `add barracks who=1 190,82`                 K 1/2007, (36480, 15744),
#                                                   on the east lake's west
#                                                   shore; `add` joins no city
#   612 `add citizen who=1 198,82`                  C 1/7
#   614 `add trireme who=0 181,82`                  T 0/6, in the east lake
#   616 `add hoplite who=0 204,88`                  0/7..0/9, on who=1's land
#   620 `be 1` / `@flight 1 2006 1 6`, 621 `be 0`   P home. The DLL issues only
#                                                   for `console->play` (its
#                                                   refusal 1); the pump stamps
#                                                   the package with the seat
#                                                   as it walks it, and
#                                                   `process_group` drops a
#                                                   group not that player's —
#                                                   so `be 0` waits a frame
#                                                   (run436, below)
#   630 `@attack 0 2007 1 6`                        T at K
#   700 `@attack 0 2002 1 7`                        0/7 at who=1's Farm 1/2002,
#                                                   a member of who=1's City:
#                                                   `city_flags |= 0xe`
#   900 `be 1` / `@repair 1 2007 1 7`, 901 `be 0`   C repairs K
#
#   run437 (item 1182):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch41 \
#       --map 14 --end-frame 1770 --log-window 605 1770 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter41.cmd
#   (cover=0 on the click-free lane; run438 is the same script at cover=1
#   on the queue lane, for `UnitData::get_speed@006086f0` and
#   `ObjectData::is_siege@0046ef90`, the two NEVER rows it names.
#   run436 was the first take, `be 0` on the issuing frame and a Catapult
#   at K: both who=1 commands dropped at the pump, `docs/RUNS.md`.)
#
# The window is 1,166 blocks: the walk's last scheduled event is C's
# repair ending near 1517; 1770 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §50 has them whole).
#
# - The sortie (`Object::do_launch@0064f3b0`, `64f81c`..`6508a7`): a base
#   with a plane inside, past the chain walk, whose owner is not human, on
#   `(frame + id) % 32 == 0`: the best of every live leader's cities with
#   `city_flags & 3 == 3` (alive AND under attack) that the owner is not at
#   peace with, `(damage/100 + 1000) × (2 if an ally) / (dist/768 + 1)`;
#   each air unit inside cleared, and patrolled over it when `dist <
#   get_speed(it, 1) × mana(it)` — `UnitData::get_speed@006086f0`'s one
#   call — and, over a friend's city, only a Biplane-line plane.
# - The spill (`Object::do_damage@0064a480`, `64c10c`..`64c4e3`): a hit on
#   a building, round its cell, on each enemy of the attacker whose front
#   order is BUILD_AT or REPAIR on that building, within the building's
#   half-extent: air none; a SEA attacker asks `ObjectData::is_siege@
#   0046ef90` of itself (`64c3b3`) and a siege one spills nothing; a land
#   siege type `count / 4`; else, within `max(max_range × 192, 0x180)`,
#   `count / 8`, quiet.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run436's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 621: P a STRAFEORDER home, `returning`; 741: P inside H, no order.
# - 631: T an ATTACKORDER on K; its rounds from 649, one hit point each.
# - 722: who=1's City under attack (0/7's first blow on the Farm).
# - 778: the sortie — P an AIRPATROLORDER over who=1's City; 808: out.
# - 1078..1517: C at K on its REPAIRORDER, a spill (T's, /8) on 1128,
#   1209, 1248, 1341, 1409, 1489.
# - 1320: P home again; 1470: out on the next sortie.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. `be 1` held across the pump does not let the commands through: no
#    STRAFEORDER on P from 621, no REPAIRORDER on C from 901.
# 2. P lands and no sortie follows: P inside H with no order past 778.
# 3. The sortie over the enemy's city, or over none: P's patrol point not
#    who=1's City (42336, 16224).
# 4. No spill: C's `damage` moves only by its own heal while it repairs
#    under the rounds.
# 5. The spill's share: a trireme's hit on C an eighth of its hit on K.

0 !ai off
606 add airbase who=1 212,104
608 add biplane who=1 212,118
610 add barracks who=1 190,82
612 add citizen who=1 198,82
614 add trireme who=0 181,82
616 add hoplite who=0 204,88
620 be 1
620 @flight 1 2006 1 6
621 be 0
630 @attack 0 2007 1 6
700 @attack 0 2002 1 7
900 be 1
900 @repair 1 2007 1 7
901 be 0
