# Golden record, chapter seventy-two — the Senate's second government on a
# human: Monarchy researched at a Senate that already trained The Despot, so
# `Build::finished`'s Senate tail takes its `set_type` arm — a patriot
# already born is converted in place (item 2448, `docs/GOLDEN.md` §89;
# `senate_gov_hero`'s second arm, diff-backed before only on Great Sahara's
# AI Senate).
#
# THE GATE (parked 2437). Chapter seventy-one's `@queueup 0 625 1 2007`
# was refused by `Build::queue_up`'s tech arm: Monarchy is AGE 3 with
# PREQ1 "Gunpowder Age" in `techrules.xml`, and `type_avail` answers
# unavailable below it. The console's `age who=0 3` lifts it and nothing
# else (TECH.md: a later start age grants no epochs).
#
# THE STAGING, walked by this crate on run492's start (RON_STAGE=ch45:
# chapter72:1770, RON_STAGE_ALL): chapter seventy-one's lines whole, then
# - 1020: `age who=0 3`, 1021: Monarchy (625) queued at the Senate.
#   (run859's first take had the age on 1012: the original read it BEFORE
#   the birth, 109 hits for The Despot, where this crate's walk applies a
#   line on 1012 after it, 659 — a one-frame edge this chapter steps
#   clear of; see GOLDEN §89.)
# - 1011: The Despot `0/6` born at 659 hits (chapter seventy-one's word).
# - ~1722: the research finishes (JOB_TIME 700, from 1021); on this crate's walk the
#   Despot becomes The Monarch, same slot `0/6`, **109 hits**.
#
# WHAT WOULD FALSIFY IT. A call on trace frame F is on block F+2.
# 1. No Monarch on 1714 (or a second unit): the tail's arm or the research
#    time; the Despot standing past 1714: `set_type` is not taken.
# 2. 659 hits kept across the conversion: the arm re-derives hits.
# 3. The Despot dead or gone before 1714: not a conversion.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch72 \
#       --map 14 --end-frame 1770 --log-window 600 1770 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter72.cmd
#
0 !ai off
600 library who=0 1
601 resource who=0 all +5000
602 add senate who=0 14,74
604 tech who=0 forage on
605 tech who=0 supply on
606 tech who=0 logistics on
607 tech who=0 tactics on
608 tech who=0 operations on
609 tech who=0 strategy on
610 @queueup 0 623 1 2007
1020 age who=0 3
1021 @queueup 0 625 1 2007
