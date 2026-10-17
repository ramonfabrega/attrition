# Golden record, chapter ninety-one -- the Sniper's and Counterintelligence's
# target arms (item 2971, `docs/GOLDEN.md` §115; `SpellTypeData::
# is_valid_target@006763c0`'s 0x281 and 0x277 arms, refused by this crate
# until this chapter).
#
# THE CASTERS (`craftrules.xml` / `unitrules.xml`): the Sniper (`fbm`,
# craft 0x281 = 641) reaches 12 tiles against the Commando's sight of 8, so
# it arrives at range with the target unseen (chapter ninety's Informer
# arm, for a second caster); Counterintelligence (`febchm`, 0x277 = 631)
# is cast by a Spy on a friend it can see, or an enemy Spy.
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter91:1200`;
# `add` coordinates snap to land):
#   arm 1, the Sniper at range: Commando `0/6` is sent at the who=1
#     Hoplite `1/6`, a squad 5,500 units off.
#   arm 2, Counterintelligence on an enemy Spy (a Spy is CLOAKED, so the
#     owner never sees it: the cast is refused at `is_valid_target`'s seen
#     test; the arm's accepted targets, an infiltrated friend or ally,
#     need the other seat to cast and are parked): Spy `0/7` at the who=1
#     Spy `1/9`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch91 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter91.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add commando who=0 100,100
608 add spy who=0 120,130
616 add hoplite who=1 134,100
618 add spy who=1 134,130
620 @spell 0 641 6 1 25728 19200 6
621 @spell 0 631 9 1 25728 24960 7
