# Item 3937: parked 3932a's re-timed staging, recon3937-2 (docs/AI.md §398). killgroup3932.cmd's cast to 626 (U 0/6,
# V 0/9, W 0/12; ten fillers 0/15..0/42 take slot 0 and 2..10), with no plain move and no formation move: the attack
# on the unseen e lays a move to a point a few cells off (2568,19896), so recon3937-1's GM X had nothing to walk.
# `@gather` onto the Woodcutter's Camp 0/2001 (4224,28608) is the no-0x04 group move with a far destination
# (run1070): slot 11, id 640900, V 0/9 the leader. Twenty idle hoplites 0/45.. stand on the route. V's first touch,
# on 768 with 0/73, ungroups it into MOVEORDER flags 1 with no search (start_dist 0 to 1000): 3932a's falsifier.
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
