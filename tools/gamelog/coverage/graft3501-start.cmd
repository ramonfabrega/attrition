# Item 3501: the golden start (map 14, the Nubian human, tribe 4; computer tribe 11) at cover=1 on the click-free pool:
# `TypeData::type_graft@00472a50`, the `+0x68` slot of every non-unit type's `Type` sub-table, which runs only when
# `LeaderData::get_graft@006e30f0` finds `tribe_can_type` 0 on a build type. No shipped building's TRIBE_MASK clears
# the Nubians' bit, so the script host clears it: `disable_type_by_tribe` (run1092 proved the `!eval` door with two
# names that match nothing), then `train_unit` grafts the unit's `where` (Hoplites -> Barracks) through `get_graft`.
# `who` is 1-based in every body (who 1 = leader 0, the human). The golden start has no Barracks, so `find_build`
# finds none and nothing is queued: the Barracks bit is the only state these lines change.
# 676/678 retry with the `rules.xml` KEY in case `Tribes::find` misses on the English name (its loader is not read).
# `+0x68` read off the image before staging (Loop 3500): BuildTypeData, BuildTypeOut and BuildType's `Type`
# sub-tables all hold 0x00472a50 there.
# The twin is this file at cover=0: every input shared, so there is no `--from`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1134 \
#       --map 14 --end-frame 690 --log-window 664 690 --timeout 3600 --detail end:LEADERS=2 \
#       --cmd-file tools/gamelog/coverage/graft3501-start.cmd --cover cover=1
0 !ai off
670 !eval disable_type_by_tribe("Barracks", "Nubians")
672 !eval train_unit(1, 1, "Hoplites")
676 !eval disable_type_by_tribe("Barracks", "nubians")
678 !eval train_unit(1, 1, "Hoplites")
