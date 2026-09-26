# Golden record, chapter eighteen — the build line: an issuer the AI takes
# through its own planner.
#
# docs/GOLDEN.md §26 (item 779; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, one more verb: `@build <who> <x> <y> <type> <o>…` is a
# call, from rontrace.dll, of the original's own `CommandManager::
# issue_build@00941c30(group, x, y, x, y, type, QUEUE_NEW 2)` — what
# `Options::picked_spot@00721c40:531` passes through `GroupOut::
# issue_build@00708c60` for an unmodified drop with no drag.
#
#   run241 (item 779):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch18 \
#       --map 14 --end-frame 1450 --log-window 605 1450 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter18.cmd
#
# `GUYS=4` with `GROUPS=1`: run215's, run219's and run223's levels, the
# line whose pool printed (run210 at `GUYS=2` did not). `LEADERS=2` prints
# the buckets the price comes out of; `BUILDS=7` prints each site's
# `(int)construct_hits`, `frame_started`, `helpers` and `build_masks`. The
# window runs to 1450: the parting this crate is expected to show is 621,
# the Barracks is expected finished near 1160, and the builders' next
# orders after it are the last thing the chapter reads.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 779, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects
# 6–9 as live captains and 5 as a non-captain, `issue_build(group, x, y,
# x2, y2, type, queued)` appends a 25-byte `build` (type 0x19): `[x][y]
# [x2][y2][type][queued]`, each as passed, behind a fresh `group` — 5 bytes
# for one citizen (30 in all), 9 for three (34) — or the 3-byte reuse for a
# repeated selection (28). A non-captain is dropped (7 bytes for [7, 5, 8]).
# `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
# nothing. The issuer tests neither the point, the type nor the builders.
# It writes the package, `CommandPackage::last_who_sent`, `last_num_sent`,
# the `last_objects_sent`/`last_uids_sent` lists (through the fixture's
# memcpy adapter) and the exception chain: **no site, no price, no order,
# no draw.** What makes a class is at process time:
# `CommandPackage::process_build@00948110` -> `Group::action_build@
# 00707510` -> `Group::action_swarm_around@0070fbe0`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §26; docs/ORDERS.md §5.1–§5.5).
#
# 1. **`action_build` places the site and pays for it, once.** It needs
#    the group on the map, `GroupData::validate_build@00708620` (the snap,
#    `blocked_site` clear, the price affordable), `num_valid > 0`, the city
#    limit for a city, `action_begin` (`+0x28 = 0`), and `LeaderData::
#    type_avail(type, 1) == 4`. Then its one `PathData` — the first point;
#    the second is never read — is popped once: `snap_center`,
#    `blocked_site`, the price (`+0x84` can-afford, `+0xd4` pay), and
#    `Objects::init_build(who, type, x, y, 0, −1)`, the site unstarted.
# 2. **The builders are `action_swarm_around(site, who, QUEUE_NEW,
#    BUILD_AT, 1)`**, members in two passes (land, then sea; air never)
#    to `group.num`. For each citizen: a ring spot `find_nearby_spot(site,
#    R)` with `R = min(xs, ys) × 0x60 + 0x30` (432 for a 4×4), pushed
#    0x30 off the site on each axis and re-validated; then
#    **`Unit::add_move_facing_order(spot, facing, local_40, 0, QUEUE_NEW,
#    0, …)`** and **`Unit::add_build_order@005e5210(site, who, QUEUE_LAST,
#    1)`**. `local_40 = ~(leader_flags >> 1) & 2 | 1` for BUILD_AT: **1
#    (MOVE_TO) for a human** (`leader_flags & 4`, `LeaderData::is_human`'s
#    whole body, read inline) and 3 (EXPLORE_TO) for a computer. So each
#    citizen holds **a `MOVEORDER` (flags 0: not pathed, no action bit)
#    and a `BUILDORDER` (type 6, flags 4) on the site**, and the list it
#    held before is cleared (QUEUE_NEW).
# 3. **The walk and the site** are `Unit::do_build@005eebf0`: a builder
#    not adjacent kills its order and re-swarms at QUEUE_FIRST (the same
#    `local_40`: MOVE_TO for a human); adjacent, it sets `CHAR_BUILD`,
#    faces the site and constructs `ACCEL_CONSTRUCT` (100) a frame into
#    `Wall::do_construct` (docs/CITIES.md §3.3).
# 4. **Finished**, a non-gather building's builder with nothing queued goes
#    to `Unit::build_done@00603bf0`: a human of stance 1 tries
#    `find_build_spot` (a friendly unfinished site, fewest builders), then
#    `find_gather_spot`.
#
# ---------------------------------------------------------------------------
# THE CAST, on who=0's own open ground south-east of Napata (run223's start
# `WORLD`: cells x 5–10, y 44–50 are BASELAND owned by 0, no feature).
# `library who=0 2` holds The Art of War (Military level 0), so both types
# are available, and leaves who=0 240 timber, 113 wealth, 100 metal on 619.
#
# - four citizens: `0/6` at tile (24, 184); `0/7`, `0/8`, `0/9` at tiles
#   (24, 196), (26, 196), (28, 196).
# - 620: `0/6` alone drops a **Barracks** (427, 4×4, 120 timber) at
#   (7296, 34176), cell (9, 44): **`0/2007`**, who=0's first free id.
# - 640: the three drop a **Siege Factory** (430, 4×4, 60 timber and 60
#   metal) at (7296, 36864), cell (9, 48): **`0/2008`**. Two types so the
#   second price does not rise on the first's count (a second Barracks is
#   145 timber against 122 in hand).
#
# A call on trace frame F is processed between F+1 and F+2 and is on block
# F+2 (§17).
#
# ---------------------------------------------------------------------------
# THE PREMISE'S KILLER, and its writers (§3, point 5; parked 667).
#
# The killer is **each commanded citizen's order stack on its processed
# block**: anything but a `MOVEORDER` then a `BUILDORDER` (flags 4) on the
# new site. The class's one constructor on this path is `add_build_order`'s
# `get_obj(BUILD_AT)`; its other callers are `check_build_order`
# (QUEUE_FIRST, 1), `Wall::process`'s AI recruiter (QUEUE_NEW, 0) and
# `come_out` (QUEUE_LAST, 0), none reached by a staged human. The approach
# class is `add_move_facing_order`'s switch on `local_40`. The order is
# killed by `do_build` (a dead or finished site, or a builder not
# adjacent) and by `check_build_order`. The loops: `action_build`'s
# `PathData` stack, one push; `action_swarm_around`'s two passes over
# `group.num` (1 and 3); `find_nearby_spot`'s ring to `R`.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The issue does not reach the pump.** Trace frames 620 and 640: an
#    INFO 17 with a refusal, or no `process_build` on 621 and 641
#    (COMMANDMANAGER).
# 2. **No site, or not paid.** Block 622: no `BUILDDATA` 0/2007 with
#    `orig_type 427` at the snapped point, unstarted, or timber not down by
#    120; block 642: no 0/2008 `orig_type 430`, or timber and metal not
#    down by 60 each.
# 3. **The class is not a build, or the approach not a move.** Block 622
#    for `0/6`, 642 for `0/7`–`0/9`: anything but exactly `MOVEORDER`
#    (flags 0) then `BUILDORDER` (flags 4, `ox` the site, `whom 0`). An
#    `EXPLORETOORDER` there says `local_40` is not the human flag.
# 4. **The walk does not end at the site.** No builder adjacent and
#    constructing (`frame_started` set, `construct_hits` rising) by block
#    760; or a second approach order (a QUEUE_FIRST re-swarm) on any.
# 5. **The rate is not one builder's, or three's.** The Barracks under
#    `0/6` alone not finished by 1200; the Siege Factory under three not
#    finished by 1000.
# 6. **The builders do not let go.** Past its site's last frame of
#    construction, a builder still holding its `BUILDORDER`.
#
# ---------------------------------------------------------------------------

0 !ai off
600 library who=0 2
610 add citizen who=0 24,184
612 add citizen who=0 24,196
614 add citizen who=0 26,196
616 add citizen who=0 28,196
620 @build 0 7296 34176 427 6
640 @build 0 7296 36864 430 7 8 9
