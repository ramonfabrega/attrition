# Golden record, chapter fourteen — the formation line, an issuer the AI
# never uses.
#
# docs/GOLDEN.md §22 (item 723; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, one more verb: each `@form` line below is a call, from
# rontrace.dll, of the original's own `CommandManager::issue_form@00941580`
# with a formation button's arguments and no modifier held
# (`Options::do_formation@007215b0` through `GroupOut::issue_form@0070b090`:
# the button's formation, rotate 0, QUEUE_NEW). The turn pump processes the
# command it appends as it processes a click (tools/trace/tracer.c,
# `issue_line`). The `@move` between them is chapter nine's right-click. No
# capture on disk holds a formation command: the console has no formation
# verb (docs/RUNS.md run45), and every group on disk is the AI's or a
# right-click's.
#
#   run210 (item 723):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch14 \
#       --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter14.cmd
#
# `GROUPS=1` puts the whole 512-slot pool in every block; `DEATHS` is OFF
# under `[End Frame]` because `dump_deaths@0092fd80` would leave the logger's
# type at WORLD and the pool's lines would fail `check_accept` silently
# (docs/ORACLE.md, run30 and run166). With it off, `dump_units` is the
# dumper `GameLog::full_dump@00930380` runs just before `dump_groups`, as in
# run178's line. Nobody dies on this ground.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 723, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects
# 6..10 as live captains:
#   `issue_form(group [7], form 2, rotate 0, QUEUE_NEW 2)` appends **18
#   bytes** — the 5-byte `group` (num 1, who 0, the one object) and a
#   13-byte `form`, type 0x03, `[form i32][rotate i32][queued i32]` — and
#   writes the package's size and data and the selection caches
#   `CommandPackage::last_who_sent` (00c06354) and `last_num_sent`
#   (00cae5fb), and `last_objects_sent`/`last_uids_sent` through the
#   imported memcpy. Nothing else: no unit, no order, no draw. The same
#   selection again appends the 3-byte reuse (16); `form -2, rotate
#   0x40000000, queued 1` and `form -1` ride through as passed; a
#   non-captain in the list is dropped. `use_mp_playback`, `semaphore &
#   0x10` and `semaphore & 4` each append nothing. The prologue is
#   issue_follow's to the byte (`sub esp, 0x10`).
# **What the emulator cannot reach**: `CommandPackage::process_form@00949d90`
# -> `Group::action_form@00707220`, and under it `Group::set_up_insert@
# 0070e520`, `action_halt@0070d0c0`, `GroupData::get_loc_to@0070c5d0`,
# `Group::action_move_to@0070fba0` -> `action_move_near@00704990` ->
# `Form::compute@0072e8e0`, and `Group::finish_insert@0070e620`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §22; docs/GROUPS.md §6; the decompile of each
# function named).
#
# **No `FormOrder` is made.** `action_form` never asks for one: nothing in
# the export calls `OrdersMemManager::get_obj(CHANGE_FORM)`; `get_new_order`
# builds one only for the save loader (`OrderList::walk_data`,
# `get_new_data`) and for `copy_order`, which copies an order that exists.
# The formation is a byte on each unit and a group move.
#
# `process_form` hands `form, rotate, queued` to `action_form(g, form,
# rotate, queued, g, 0)`. For a unit group on the map whose leader is no
# plane, with an explicit formation and QUEUE_NEW:
#   group `form` (+0x10) = -1; `set_up_insert` copies the leader's
#   action-bit orders aside; `action_halt(0)`; recurse with QUEUE_NEW when
#   nothing was copied, QUEUE_FIRST when something was, `param_5 = 1`;
#   `finish_insert`.
# The recursion writes `unit +0xaa = form` on every member that is a unit
# and no plane — figures included, citizens NOT exempt here — and then:
#   QUEUE_NEW (nothing was copied: the group stood): `get_loc_to` — the
#     leader's final point, which for a unit with no orders is where it
#     stands — and `action_move_to(that, QUEUE_LAST, set_angle 0, angle 0,
#     MOVE_TO, action 0, form -1, width -1, disembark 0)`. `form -1` takes
#     `get_form()`, the byte just written. The group re-forms **on the
#     spot**, each member walking to its slot round the leader, and the
#     orders carry **no action bit**.
#   QUEUE_FIRST (the group was walking): nothing more; `finish_insert`
#     replays the copied `GroupMoveOrder` (case 0x13) as `action_move_near(
#     orig_x, orig_y, tolerance, QUEUE_LAST, set_angle 1, the order's own
#     angle, MOVE_TO, action 1, form -1, …)` — the same destination, laid
#     out again in the new formation, with the action bit.
#
# ---------------------------------------------------------------------------
# THE CAST, on chapter thirteen's ground (cells x 0..6, y 14..22: BASELAND,
# no border, run204's and run208's start WORLD; no Barracks here). Three
# squads, so the formation has three captains and two categories:
#   0/6..0/8   Hoplites, captain 0/6   (FORM_CAT_FOOT; the leader)
#   0/9..0/11  Hoplites, captain 0/9   (FORM_CAT_FOOT)
#   0/12..0/14 Slingers, captain 0/12  (FORM_CAT_FOOT_RANGED)
# Formation 2 is Envelop and 0 is Line (`rules.xml`'s order; both are
# formation buttons, 0..4, the set `get_form_option` counts).
#
# A call on trace frame F is processed between blocks F+1 and F+2, and its
# orders are in block F+2 (docs/GOLDEN.md §17's convention).
#
#   620: the group stands; Envelop.       Its orders are in block 622.
#   700: the right-click, north.          Envelop is what the move reads.
#   740: Line, on the move.               Its orders are in block 742.

0 !ai off

610 add hoplite who=0 12,84
612 add hoplite who=0 18,84
614 add slinger who=0 15,88

620 @form 0 2 0 6 9 12
700 @move 0 2976 12000 6 9 12
740 @form 0 0 0 6 9 12
