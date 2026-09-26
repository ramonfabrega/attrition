# Golden record, chapter twenty-one — the repair line: a player's
# right-click on a damaged building of its own.
#
# docs/GOLDEN.md §29 (item 813; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, one more verb: `@repair <who> <ox> <whom> <o>…` is a call,
# from rontrace.dll, of the original's own
# `CommandManager::issue_swarm_around@009416b0(group, ox, whom, QUEUE_NEW 2,
# REPAIR 13)` — what `Console::execute_at_cursor@007c6630` passes through
# `GroupOut::issue_swarm_around@0070afc0` for an unmodified right-click on a
# damaged, finished building of one's own, and what `Options::picked_spot@
# 00721c40` passes for the Repair pick. The `repair` command (0x10) is
# issued by nothing.
#
#   run255 (item 813):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch21 \
#       --map 14 --end-frame 1300 --log-window 605 1300 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter21.cmd
#
# `GUYS=4` with `GROUPS=1`: every golden capture's level since run215.
# `AMMO=5` prints each arrow's launch and landing (docs/COMBAT.md §22), the
# damage's source; `BUILDS=7` the Barracks' `damage`, `damage_frac` and
# `helpers`; `UNITS=3` each citizen's stack and `unit_masks`; `LEADERS=2`
# the buckets the repair's price comes out of. The window is 695 blocks:
# this crate's last event is `0/9`'s order dying on 1023, and 250 blocks of
# runway past it end at 1273.
#
# ---------------------------------------------------------------------------
# THE ISSUER, under the emulator before the pair (item 813, step 1).
#
# On command_oracle.py's fixture, `issue_swarm_around(group, 2006, 0, 2, 13)`
# appends a 17-byte `swarm_around` — type 0x06, `[ox i32][whom i32][queued
# i32][orders i32]` — behind the 5-byte `group`, 22 bytes, or 20 with the
# 3-byte reuse; `ox` or `whom` negative appends nothing, and so do
# `use_mp_playback` and `semaphore & 0x10`. It writes the package and the
# selection caches: no unit, no order, no draw. Its prologue is
# issue_attack's, `55 8b ec 83 ec 14 56 8b 75 0c c6`.
#
# THE STAGING, walked before the capture (run256, this file's twelve lines
# to 830, four takes; docs/RUNS.md run256).
# - Take 1: the Barracks on neutral ground is no target — `check_target`
#   wants `build_flags & 0x10` or an owned cell — so the hoplites never
#   struck it. The command was processed all the same.
# - Take 2: in who=0's territory the hoplites strike (4 a blow), but this
#   crate parts on 636 on their first step to it (a walk animation's draw,
#   `movement.rs`/`anim.rs`), so the attackers became archers who stand.
# - Takes 3 and 4: who=1's Bowmen, piece 120, launch from bays nobody had
#   measured; take 4's twelve arrows measured them (`sim::launch`, `BAYS`),
#   and this crate then walks take 4 whole to 830.
# What take 4 printed: the arrows from 644, the Barracks' `damage` 0 → 3
# (frac 12) by 744; `peace 1` on 760 and the Bowmen walking off from 765;
# `process_swarm_around 2007 0 2 13 781`; on 782 `0/6`'s `unit_masks` 0 →
# 1034 (the `0x400` builder bit) and its stack a `MOVEORDER` (flags 1) to
# (5736, 33096) with a `REPAIRORDER` (flags 4, `ox 2007`, `uid 13`) behind
# it; `process_swarm_around 2007 0 2 13 801` and the same on `0/7`..`0/9`
# on 802.
#
# ---------------------------------------------------------------------------
# THE CAST, in who=0's territory beside Napata (docs/GOLDEN.md §4's free
# ground, tiles 24–35, 160–179).
#
# - 600 / 602: `library` 2 for who=0 and 3 for who=1.
# - 606: a who=0 **Barracks**, `0/2007`, at tile (30, 170), (5760, 32640).
# - 610: who=1 **Bowmen** at tile (34, 170): `1/6`, `1/7`, `1/8`.
# - 760: `peace 1` — the Bowmen stop, and the building is not under attack.
# - 770–776: four who=0 citizens, `0/6`..`0/9`, twenty tiles south.
# - 780: `@repair` on `0/6` alone; 800: on `0/7 0/8 0/9` together.
#
# A call on trace frame F is processed between F+1 and F+2 and is on block
# F+2 (§17).
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (docs/GOLDEN.md §29).
#
# 1. The issue does not reach the pump: trace frame 780 or 800, an INFO 17
#    with a refusal; or no `process_swarm_around 2007 0 2 13` on 781 or 801.
# 2. The command is not the swarm: on block 782 (802 for the trio) the
#    citizen's stack is not a `MOVEORDER` to a ring spot with a
#    `REPAIRORDER` (flags 4, `ox 2007`) behind it — a lone `REPAIRORDER`
#    (the `repair` command's `action_repair`), an `EXPLORETOORDER`
#    approach, or a `BUILDORDER` each kills it; or `unit_masks & 0x400`
#    not set on 782.
# 3. The repair does not happen: `0/2007`'s `damage` still above 0 ten
#    blocks after `0/6` stands at its spot (this crate: arrives 929, 3 → 1
#    → 0 on 930–931).
# 4. The late repair lives on: `0/7`, `0/8` or `0/9` still holding a
#    `REPAIRORDER` the block after it arrives at an undamaged Barracks
#    (this crate: 968, 969, 1023), or taking a `GATHERORDER` there.
# 5. The three spots are not three: two of `0/7`..`0/9`'s `MOVEORDER`
#    points equal on 802.
#
# ---------------------------------------------------------------------------

0 !ai off
600 library who=0 2
602 library who=1 3
606 add barracks who=0 30,170
610 add bowmen who=1 34,170
760 peace 1
770 add citizen who=0 28,190
772 add citizen who=0 26,192
774 add citizen who=0 28,192
776 add citizen who=0 30,192
780 @repair 0 2007 0 6
800 @repair 0 2007 0 7 8 9
