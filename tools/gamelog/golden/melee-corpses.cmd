# The sixth melee -- the third's lobby with two massed armies, past 100 live corpses (item 3971, GOLDEN 177; the shape is GOLDEN 155).
# Lobby: run1467's, held whole -- map 7, --profile PLAYERS=2 STARTING_TECHNOLOGY=7 ENDING_TECHNOLOGY=7 STARTING_RESOURCES=7,
# --seat-tribe 1=10 --seat-tribe 2=13; who 0 the idle human (Nubians, tribe 4), who 1 the French (10), who 2 the Russians (13).
# No lever moves (GOLDEN 155): the subject is `Objects::add_death@00653b60`'s pool of 100 `DeathObj` slots, which no
# capture on disk fills (the archive's busiest block holds 34 records). `add N` is N units (`run_cmd` case 0x4e, capped
# at 300); the armies stand two tiles apart, who 1 on tile row 80 and who 2 on row 82 (the artillery a row behind), so
# every line is in range at once: on the third melee's rows, ten tiles apart, ours peaked at 44 records.
# No `!ai off`, no `select`, no issuer, no `tech all` (GOLDEN 149).
# The killers, written before the take (run1542, stride 50; ours on the staged walk fills the pool from ~980 to ~1250):
#  1. No stride block prints more than 100 DEATH_OBJS records. More kills the pool reading (a pool that grows).
#  2. No block reaches 100: the staging failed its premise, and the take says nothing of the pool.
#  3. A record leaves before its life is out on a block that held 100: it must be the oldest non-blocking record
#     (lowest first_frame, lowest slot on a tie). A younger one leaving first kills the eviction order.
600 resource who=1 all +100000
602 resource who=2 all +100000
610 add 100 elite_special_forces who=1 80,80
612 add 100 elite_special_forces who=2 80,81
614 add 40 advanced_machine_gun who=1 84,80
616 add 40 advanced_machine_gun who=2 84,81
618 add 10 rocket_artillery who=1 88,79
620 add 10 rocket_artillery who=2 88,82
