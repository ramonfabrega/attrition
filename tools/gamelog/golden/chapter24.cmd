# Golden record, chapter twenty-four — the queue line: a Barracks' infinite
# queue toggled on between two finishes.
#
# docs/GOLDEN.md §33 (item 877; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot
# and a Hoplite squad, `!ai off`), then the player's production:
#   620 `@queueup 0 132 1 2007`   Hoplites (type 132), one, at the Barracks
#   640 `@queueup 0 170 1 2007`   Bowmen (type 170), one, behind them
#   900 `@buildmask 0 64 2007`    the infinite-queue button (0x40), between
#                                 the Hoplites' finish and the Bowmen's
#  1300 `@buildmask 0 64 2007`    the same button on an empty queue
# `@queueup` is the DLL's call of `CommandManager::issue_queue_up@00941be0`
# (verb 18, new here); `@buildmask` is chapter twenty-three's verb 17 with
# the mask 0x40 (`Options::exec@007188c0`'s option 0x40 on a selection of
# buildings).
#
#   run285 (item 877):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch24 \
#       --map 14 --end-frame 1560 --log-window 605 1560 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter24.cmd
#
# The window is 955 blocks. The last falsifier is the empty-queue toggle on
# 1302; 1560 leaves 258 blocks past it.
#
# ---------------------------------------------------------------------------
# THE ISSUERS, under the emulator before the capture (item 877, step 1).
#
# On command_oracle.py's fixture with a building (`Build::vftable`) in
# who=0's registry:
# - `issue_queue_up(group [b], type, num)` appends the 5-byte `group` and a
#   9-byte `queue_up` (0x18, `[type i32][num i32]`), 14 bytes, each as
#   passed. It tests nothing of the type.
# - `issue_buildmask(group [b], 0x40, set)`: 14 bytes, `set` 1 whatever the
#   third argument (chapter twenty-three's finding, again).
# - `WallData::valid_buildmask@0063e2a0(0x40)` answers 1 only when the
#   building's vslot 0x20 answers (1 on `Build`) and `BuildData::
#   can_infinite@0062d4d0` does: `BuildTypeData::is_training_building`
#   (`build_flags & 0x80000000`) and **a train job in the queue** — a unit
#   type (0x32..0x19d) whose availability bit is set. It answers 0 on an
#   empty queue, on a queue of research entries (the bit clear), on a tech
#   entry, and on a building that is not a training building.
# - `Group::action_buildmask@006fc9a0(0x40, set)` on an admitted Barracks:
#   4104 -> 4168 and 4168 -> 4104 with `set` 1 or 0; 4296 -> 4232 (the
#   0x80 left alone). With only a research entry, or an empty queue, it
#   writes nothing.
# - `Build::unqueue@006207c0(0, 0)` on a one-entry queue with 0x40: the
#   queue empties and the bit is cleared (0x1048 -> 0x1008); on a
#   two-entry queue the bit stays.
# What the emulator did not reach: `Build::do_queue@0061e410`'s completion
# arm (read from the listing below), `Group::action_queue_up@006fdbb0`, and
# `process_group`'s push of each building group, at process time.
#
# ---------------------------------------------------------------------------
# THE READERS OF `build_masks & 0x40` (`+0x60`, grepped in every spelling,
# parked 869). In the simulation:
# - `Build::do_queue` at `61ec24`: after `finished` answers > 0, the word
#   is read, then `unqueue(i, 0)`; if the entry was a train job (the bit
#   was set before `finished`) and the word had 0x40: `&= ~0x40`,
#   `queue_up(type, 0)`, and on success (0) `|= 0x40`. So the re-queue goes
#   to the end of the queue, is paid, and a refusal leaves the bit off.
# - `Build::unqueue` clears it when `queued` reaches 0; so do
#   `Build::clean_queue@00620b60` (close, capture, defeat) and
#   `Build::action_unqueue@00620280` (the player's cancel, which clears the
#   bit and returns without removing anything for a single cancel).
# - `ScenarioFuncSet::toggle_infinite_queue@009f45e0`: out of v1.
# The rest are the interface's: `IFaceSelected::update_queue`,
# `IFaceOptions::setup_button`, `GroupData::get_infinite_queue`, and
# `GroupOut::issue_queue_up`'s queue-full message. **No AI function reads
# it.** (`& 0xffbf` in `come_out`, `action_alarm`, `Wall::start_me`,
# `BuildType::mask_me` and the Cliffs is a city flag or a tile mask.)
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run208's start (chapter thirteen's
# capture, which is this game to 619), with `Sim::queue_up` by hand on the
# processed frames and the re-queue emulated. No staging run: run286 was
# not used.
#
# - 619: who=0 holds 253 food, 240 timber, 113 wealth; pop 8 of 25. The
#   Barracks trains Scouts, Slingers, Hoplites (132) and Bowmen (170),
#   each with its bit set: every entry here is a train job.
# - 622: `[132]`, job_counter 100, food 204, timber 204.
# - 642: `[132, 170]`, wealth 63.
# - 848: the Hoplites out; `[170]` at 0. The bit is clear: no re-queue.
# - 902: the bit set (the Bowmen are a train job).
# - 1052: the Bowmen out; re-queued, `[170]` at 0, paid 46 timber and 56
#   wealth out of 72; the bit kept.
# - 1264: out again; 24 wealth against 56: refused, the queue empty, the
#   bit off.
# - 1302: the second toggle on an empty queue writes nothing.
# The margins: the toggle is 54 blocks after the Hoplites' finish and 150
# before the Bowmen's; the first re-queue has 16 wealth over, the second
# is 32 short.
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @queueup 0 132 1 2007
640 @queueup 0 170 1 2007
900 @buildmask 0 64 2007
1300 @buildmask 0 64 2007
