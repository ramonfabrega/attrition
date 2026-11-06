# Item 3940: parked 3937a, a stash under the ungroup's copy (docs/AI.md §399). killgroup3937.cmd's recon3937-2 whole:
# `640 @gather 0 2001 6 9` lays GroupMoveOrder 640900 (slot 11, V 0/9 leading), V's touch on 768 ungroups it into
# the copy MOVEORDER flags 1 (group 11) toward (4008,28632), which V walks unobstructed up x=4008 from 796 to past
# 960 with collide 0. Added: a walker 0/105 north of that line, sent at 790 down it to (4008,21000), so the two
# movers meet head-on near (4008,24000) about 860 with V's copy current -- the mutual touch resolve_unit_collision
# :388-405 asks before its search at :482. The recon reads V's start_dist and collide_frame there.
# recon3940-1: the touch is mutual in block 895 (V collide_o 105, 0/105 collide_o 9); V side-steps with start_dist 0
# through 920 -- the search finds a path without suspending, so no stash; the pair is RESIDUE (§399.3).
0 !ai off
595 add hoplite who=0 12,104
597 add hoplite who=0 14,104
599 add hoplite who=0 13,107
601 add hoplite who=0 8,86
602 add hoplite who=0 11,86
603 add hoplite who=0 14,86
604 add hoplite who=0 17,86
605 add hoplite who=0 20,86
606 add hoplite who=0 9,89
607 add hoplite who=0 12,89
608 add hoplite who=0 15,89
609 add hoplite who=0 18,89
610 add hoplite who=0 21,89
611 add hoplite who=0 13,114
611 add hoplite who=0 14,114
611 add hoplite who=0 15,114
611 add hoplite who=0 16,114
611 add hoplite who=0 17,114
612 add hoplite who=0 13,115
612 add hoplite who=0 14,115
612 add hoplite who=0 15,115
612 add hoplite who=0 16,115
612 add hoplite who=0 17,115
613 add hoplite who=0 13,116
613 add hoplite who=0 14,116
613 add hoplite who=0 15,116
613 add hoplite who=0 16,116
613 add hoplite who=0 17,116
614 add hoplite who=0 13,117
614 add hoplite who=0 14,117
614 add hoplite who=0 15,117
614 add hoplite who=0 16,117
614 add hoplite who=0 17,117
615 add hoplite who=0 21,135
619 add chariot who=1 16,40
620 @move 0 2400 20064 6
621 add chariot who=1 13,70
621 @move 0 1632 16608 15
621 @move 0 2208 16608 18
621 @move 0 2784 16608 21
621 @move 0 3360 16608 24
621 @move 0 3936 16608 27
621 @move 0 1824 17184 30
621 @move 0 2400 17184 33
621 @move 0 2976 17184 36
621 @move 0 3552 17184 39
621 @move 0 4128 17184 42
625 be 1
625 @stance 1 2 6 9
626 be 0
640 @gather 0 2001 6 9
790 @move 0 4008 21000 105
