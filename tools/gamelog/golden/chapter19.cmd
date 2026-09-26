# Golden record, chapter nineteen — the cast line: a Spy's Informer on an
# enemy building, the first targeted craft any capture has issued.
#
# docs/GOLDEN.md §27 (item 790; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, one more verb: `@spell <who> <type> <ox> <whom> <x> <y>
# <o>…` is a call, from rontrace.dll, of the original's own
# `CommandManager::issue_spell@00941b80(group, type, ox, whom, x, y)` —
# what `Options::picked_spot@00721c40:749` passes through `Options::
# target_spell@0071dda0` for an unmodified pick of a targeted craft: the
# craft, the object under the cursor and the cursor's own point.
#
#   run245 (item 790):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch19 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter19.cmd
#
#   Run 2026-09-25: 540 s, 201 MB, the same game as run246's packet to
#   625, 253,440 GROUPDATA. The command processed on 621. No falsifier
#   fired: a CASTORDER (flags 4, paid 1) and a MOVEORDER to (14232, 15528)
#   on 622, mana_burn +500 and no bucket down; in range on 756, spell_time
#   1..39, 1/2006 infiltrated on 795; 0x20000 across 756..794, the cloak
#   kept, the mana still. docs/RUNS.md, run245.
#
# `GUYS=4` with `GROUPS=1`: every golden capture's level since run215, the
# line whose pool printed. `LEADERS=2` prints the buckets a price would come
# out of; `BUILDS=7` prints the target's `infiltrated`; `UNITS=3` prints the
# Spy's `mana_burn`, `spell_time`, `unit_masks`, `visible` and `flags`, and
# the `CASTORDER`'s `x y paid spell` behind its `TARGETORDER` base. The
# window runs to 1100: the parting this crate is expected to show is 621,
# the cast is expected near 760, and 250 blocks of runway follow it.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 790, step 1).
#
# Under unicorn on command_oracle.py's fixture, `issue_spell(group, type,
# ox, whom, x, y)` appends a 21-byte `spell` (type 0x17, `[ox][whom][type]
# [x][y]`, each as passed) behind a fresh `group` — 26 bytes for one object
# — or the 3-byte reuse (24). `use_mp_playback`, `semaphore & 0x10` and
# `semaphore & 4` each append nothing. It writes the package, the selection
# caches and the exception chain: no order, no price, no draw.
#
# THE CAST'S PREDICATES, on the original's own state (run246, a packet at
# logger frame 619 of this script without its `@spell` line; step4.py's
# machinery). `SpellTypeData::is_castable(craft, 0/6, 0, 0)` is 3 for Bribe
# (0x275), Counterintelligence (0x277) and the Informer (0x27f);
# `is_valid_target(craft, 0, 2006, 1)` is 1 for the Informer alone (Bribe
# and Counterintelligence refuse the Barracks); `get_range` 960 for it (the
# row's 1,920 halved for a building), `get_job_time` 40, `can_pay_cost` 10.
# The Spy holds `mana` 1000 and `mana_burn` 491. And `Group::action_spell`
# then one `Unit::do_cast`, entered on the packet with a pool Group of [6]:
# a **`CastOrder` (type 14, flags 4)**, the Spy's `+0xa2/+0xa8/+0xa6` set to
# the target and `spell_time` 0; then `paid` 1, `mana_burn` +500, every
# resource bucket rewritten **unchanged**, and a **`MOVEORDER` pushed ahead
# of the cast** to (14232, 15528).
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §27; docs/ORDERS.md §6.9).
#
# 1. **`action_spell` lays the order.** `validate_spell` again, then — the
#    craft carries `m` (0x1000) — the member with the most `mana_left` among
#    those that can cast it; `mana_left + pending ≥ MANA` (500); `can_pay_
#    cost`; `is_valid_target`; not busy; the target written onto the unit;
#    and **`Unit::add_cast_order@005e4a60(ox, whom, x, y, 0x27f,
#    QUEUE_NEW, 1)`** — no `g` (0x40) in the craft, so not QUEUE_FIRST.
# 2. **`Unit::do_cast@005ebfe0`, the targeted arm** (`FLAGS & 0xe`):
#    `pay_cast_costs` once (`paid`), then the target re-read (`is_valid_
#    target`); out of `get_range + radius` (960 + 384 for a 4×4), a
#    `find_nearby_spot` ring on the target at 1,152–1,296 and an
#    `add_move_order(spot, QUEUE_FIRST, tolerance 1344)` ahead of the cast.
#    In range: the target must be `is_seen(0, 0)`, the Spy `flags |= 0x80`
#    and `visible |= 1 << 1`, faces the target, `set_anim(CHAR_ATTACKWALK
#    0xa, 0, 0)` every frame, `unit_masks |= 0x20000` once; the craft's `l`
#    (0x800) keeps the cloak; `spell_time` climbs; on the fortieth frame
#    `SpellType::cast` → `cast_double_agent@00673a80`: the target's
#    `infiltrated |= 1 << 0` and its `update_seen(0)`; then
#    `kill_current_order`, which clears 0x20000.
# 3. **Mana** recovers one a frame while `unit_masks & 0x2a000` is clear
#    (`Unit::process@00610bc0`), so it stands still from the first in-range
#    frame to the cast.
#
# ---------------------------------------------------------------------------
# THE CAST, on neutral open ground in the middle of the map (chapter
# seventeen's arena; run241's start `WORLD`: cells x 12–23, y 17–22).
# `library` puts both sides in the Medieval age, where the Spy's
# prerequisite sits.
#
# - 606: a who=1 **Barracks** at tile (80, 80): **`1/2006`**, at (15360,
#   15360) on the packet.
# - 610: a who=0 **Spy** at tile (60, 82): **`0/6`**, at (11640, 15864),
#   born with `mana_burn` 500 (`Unit::init`: a spy's is `mana / 2`).
# - 620: `0/6` casts the **Informer** (0x27f, 639) on `1/2006`, the pick at
#   the Barracks' own point.
#
# A call on trace frame F is processed between F+1 and F+2 and is on block
# F+2 (§17).
#
# ---------------------------------------------------------------------------
# THE PREMISE'S KILLER, and its writers (§3, point 5; parked 667).
#
# The killer is **the Spy's order stack on block 622**: anything but a
# `MOVEORDER` then a `CASTORDER` (spell 639, `paid` 1). The class's
# constructor on this path is `add_cast_order`'s `get_obj(CAST_SPELL)`; its
# other callers are `think_spellcaster` (a human's QUEUE_FIRST
# Counterintelligence on a valid target in range — none is staged: no enemy
# Spy, no infiltrated object of who=0's), `think_fish`, the unpack and
# transport arms, none reached by a staged Spy. The order is killed by
# `do_cast` (a target no longer valid, an unseen target in range, the cast)
# and by any QUEUE_NEW order. The loops: `action_spell`'s member loops over
# `group.num` (1); `find_nearby_spot`'s ring to 1,296.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The issue does not reach the pump.** Trace frame 620: an INFO 17
#    with a refusal; or no `process_spell` on 621 (COMMANDMANAGER).
# 2. **The class is not a cast.** Block 622: `0/6` holding anything but a
#    `MOVEORDER` then a `CASTORDER` (spell 639, `paid` 1, x 15360, y
#    15360, its target `2006`/1). No order says `action_spell` refused.
# 3. **The price is not the mana.** Block 622: `mana_burn` not up by 500
#    (about 989), or any bucket of who=0 down (the emulator's rewrite was
#    unchanged; a 20-wealth, 20-timber drop says `get_cost` charges the
#    row's `COST`).
# 4. **The walk is not to the ring.** Block 622: the `MOVEORDER` not to
#    (14232, 15528); or `spell_time` rising before the Spy is within 1,344
#    of the Barracks, or not rising by block 800.
# 5. **The cast does not land, or costs the Spy.** Forty frames after the
#    first in-range frame: `1/2006`'s `infiltrated` not 1, the Spy's stack
#    not empty, or the Spy dead or gone.
# 6. **The cloak breaks.** Any block: the Spy's `unit_masks` taking 0x1000
#    or 0x10000. And the first in-range block: no 0x20000, `visible` not
#    taking 0x2, `flags` not taking 0x80; 0x20000 still set after the cast.
# 7. **The mana does not wait.** `mana_burn` falling while 0x20000 is set,
#    or not falling after the cast.
#
# ---------------------------------------------------------------------------

0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
610 add spy who=0 60,82
620 @spell 0 639 2006 1 15360 15360 6
