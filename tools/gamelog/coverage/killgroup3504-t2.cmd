# Item 3504: `Unit::kill_group_order@005e3310` through `Group::kill_group_order@00710aa0`, from
# `Unit::find_new_target@005ff6a0:71` at `do_move:105` under `do_group_move`'s leader arm (t51/draft-items-54.md F1).
# Chapter fifteen's cast to 614 (two Hoplite squads `0/6`, `0/9`, the Chariot `1/6`; run1138 had run1070's four who=1 Supply
# Wagons `1/7`..`1/10` across the walk south), then eight more who=0 Hoplite squads `0/12`..`0/33` standing across the
# walk home to the Town Center `0/2000` (16,160): the jam. The two squads attack-move onto the Chariot's spot
# (a `GROUP_ATTACK_TO`); `Object::find_nearby_target:570-580` gives the group `action_attack(…, mandatory 0,
# QUEUE_FIRST, 4)` once it is in range (`Group::is_attacking_to`). The take's later lines (`@repairins` onto the Town
# Center, the Chariot walked off north so the leader arm's 32-frame `kill_group_move` finds it past 0x900) are
# staged off the recon's dump; this file holds the lines every take shares.
0 !ai off
610 add hoplite who=0 12,84
612 add hoplite who=0 18,84
614 add chariot who=1 16,64
616 add hoplite who=0 11,106
618 add hoplite who=0 14,106
620 add hoplite who=0 17,106
622 add hoplite who=0 20,106
624 add hoplite who=0 12,109
626 add hoplite who=0 15,109
628 add hoplite who=0 18,109
630 add hoplite who=0 21,109
640 @amove 0 3192 12408 6 9
# run1139: recon3504-1 shows the acquisition in block 742 (`ATTACKORDER ox 6 whom 1 flags 4 mandatory 0` over the
# `GROUPATTACKTOORDER` on both squads; flags 20 on 743, the bit kept), the leader 1,709 from the Chariot. A line at
# 741 is processed between 742 and 743: the insert onto the Town Center, and the Chariot sent north at once, so the
# leader arm's next 32-frame check finds it past 0x900.
741 @repairins 0 2000 0 6 9
741 @move 1 3192 600 6
