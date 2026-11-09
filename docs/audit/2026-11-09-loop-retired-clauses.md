# The loop's retired clauses, as they stood on 2026-11-09

Moved whole from `docs/LOOP.md` by the seventy-sixth pass (`docs/DECISIONS.md`
entry 116), which made one driver of the commander and the steer. Nothing
here is live: the live rule for each is in `docs/LOOP.md`, "The driver".
Kept because a clause carries the case that taught it, and the next change
to the roster, the lane count or the handover should read what was tried.

## The roster, and who spawns whom (retired)


**Fan-out rules.** **Opus drives.** It is the default for a subagent and for
the session, and it carries this work end to end — implementation, diffs,
widenings, readings, adjudication under the marker discipline, the
documents and the queue. **Fable never reads blind and is never a
subagent.** It is the **steering session** — every twenty items, or sooner
when the headline has not moved for two sessions running — in the main
thread: is the tranche real, has the headline moved, what is the finish
line; then the batched ratification of the *marked rows only*,
any verdict that overturns an earlier one, a listing read where the
decompiler is wrong, the rewrites of this file and the queue, and **the
loop's own items** — tooling, guards, the queue's rules — which live in
`docs/PARKED.md`'s Loop section and are never spawned to a worker; **a
Loop item that two tranches running did not reach closes at the pass**,
its text kept in the pass's record, and its next reach files it again
(`docs/DECISIONS.md` entry 54). It writes the next opener and **spawns the
commander, and nothing else** — never a worker: one pass did, on an
opener that said "refill", and ratified a trial only the user decides
(DECISIONS 59, 61). Never Sonnet on a reading or an adjudication;
**the workers' roster is the user's, set at a steer** (DECISIONS 61):
**four lanes from the forty-sixth tranche: Opus 5.5 at `--effort
medium` on each of the two battery words' lanes, landing wide, and
Sonnet 5.5 at `--effort medium` on each of two sweep lanes** (Ramon, at
the forty-fifth steer, DECISIONS 84: the second sweep is the
token-for-CPU trade, its clock its own subagents' and no capture lane
held; three lanes stood from the forty-fourth steer, DECISIONS 83, four
from the fortieth, DECISIONS 78–82; `ccc spawn --name lane-<N> --worktree=att-<N> --model 'claude-opus-5-5[1m]' --effort medium` and
`--model 'claude-sonnet-5-5[1m]' --effort medium` — **`--worktree=att-<N>` on every spawn**: without it a
lane from the integration row runs in the integration checkout (Loop 3837: 3835's first spawn, stopped within the
minute; the sixty-fifth pass) — after three Opus
tranches measured against three Sonnet ones, DECISIONS 65 and 68; the
model and effort verified from each worker's transcript and the spawn
line — the harness's defaults differ by model, and a trial once moved
two settings for one).
**Two kill rules ride with a medium lane**: a booking gate red on its merge for what its
own lane gate passed returns that lane to Opus 5.5 at high — **a red
on a pin another landing's merge
moved is a re-pin, not a trip** (parked 1469; seven of twenty merges
moved one in the twenty-fifth tranche, Opus lanes among them) — and a
lane that lands nothing in three
attempts on one word gets an Opus successor on that word. **No pass
and no commander changes the roster**: a pass scores it lane by lane
and recommends, and the user decides at a steer. The model is said in
user-visible text each time. **A commit's trailer names the model the worker's own system prompt
names** — never one the brief dictates, and never the harness's attribution
reminder alone, which has been wrong. A commander may land a **one-clause safety fix in this file itself**
when its evidence is measured and its source named, filing a `FABLE:` row
the same day; everything else in this file waits for the pass.

## The lanes (retired: the lane rule's history, and the commander's count and clear)


**Two lanes, on Ramon's word (DECISIONS 114 §3); the count moves only
when a pass names the bottleneck a third lane would relieve.** The clause
below — add a lane while waiting holds, take it away when it does not — is
history from the seventy-fourth pass: it flipped the lane five times in
eight tranches without naming what it waited on; Loop 3996 named it (28
of 34 waiting minutes a session are the tests running), so the lever is
the gate's length. A pass reports the shape and recommends under "For
Ramon"; it does not move the count.

**Lanes are independent.** Lanes are throughput, not a pair — **one to
an open word, and the queue's handoff says how many**: each lands, is
chained and is refilled without reference to another's state, and the
only coupling between them is the merge and what the brief reserves.
**A pass adds one lane a tranche while a landing's waiting holds** —
`tools/tranche.py`'s waiting minutes a landing no more than a fifth
above the tranche before — **and takes one away when it does not** (the
fifty-fourth pass, DECISIONS 93: a third Opus lane raised waiting 34.3 →
58.8 a landing and the gate 26.1 → 41.0 a session in the same 9 h 55 two
lanes had taken, so the fifty-fifth runs two) — **and a lane the rule
took away is not re-added while the gate class that took it stands**
(the fifty-fifth pass, DECISIONS 94 §1: two lanes waited 40.5 a
landing against the fifty-fourth's 58.8, which the letter reads as
"holds, add one", but the third lane's cost was the gate on a
sixteen-core host, 26.1 → 41.0 a session, and the gate on two lanes
read 30.7 this tranche, above the 26.1 it was taken at; the lane
returns when a two-lane tranche's gate a session is under that figure,
or the gate's class is cut — the lever Loop 3608 pulls; **the fifty-sixth
pass measured that lever at zero**: the gate read 34.2 a session and
every stand-up's lane gate still ran the timed suite, because a stand-up
also adds the coverage driver's window line and two test-kit rows, which
the rule did not read — it reads them now, eight of that tranche's
twenty gates would have kept the fast suite, and the fifty-seventh's
drain measured it: **the rule fired on every stand-up and saved
nothing** — each gate's `gate-policy.json` reads 758 timed tests left
and the journals 16–19 release minutes — because the budget's file
times the data crate alone and the sim crate's five test binaries ran
whole, 680 of a stand-up's 1,044 release seconds in the pass's replay
of 3626's gate; **the fifty-seventh pass cut them** (Loop 3653,
`sim_crate_left`): a lane that touches no path under `crates/sim/`
leaves the sim crate to the booking gate, which runs it on every
merge — **measured at the fifty-eighth's drain**: seven of nine
stand-ups ran release 5.6–6.9 against 17.3–23.8 for the rest, and the
two that widened on one data-crate file each (3629, 3665) ran 18.0
and 18.3 because the cut was tied to the fast suite, so the
fifty-eighth pass reads it on the branch's paths alone, widened or
not (Loop 3676: the sim crate depends on nothing in the workspace);
**and the gate read 25.3 a session on two lanes, under the 26.1 the
third lane was taken at, with waiting 40.2 a landing against 46.5 —
so the third lane returns by this clause for the fifty-ninth**, three
Opus lanes at medium, read at its drain against 40.2 waiting and 25.3
gate, and a fifth above 40.2 takes it away again (DECISIONS 97); **the
fifty-ninth's drain read 57.3 and the lane goes again** (DECISIONS 98
§2) — and its cost was not the door: admission 2,834 s against the
fifty-fourth's 4,305 and two gate ends of thirteen past the cores, but
the residue lanes gated three to five times each on `Base moved:`,
every landing touching the rubric pin, the AI section, the test-kit
rows and the window line, 908 of 1,145 waiting minutes on the gate
class (Loop 3694); **so the lane returns when Loop 3694's lever is
built and measured on a two-lane tranche — a base move on row-list
paths alone no longer re-runs the lane gate — or on Ramon's word,
never on a two-lane gate figure alone, which 25.3 already was**; **the
sixtieth measured two lanes without the lever** — waiting 37.4 a
landing, the gate 26.3 a session, the door 1,096 s, and fifteen lane
gates past the first with every quoted overlap a row list — **and the
sixtieth pass built it** (`release_gate.py`'s `row_list_overlap`,
DECISIONS 99 §2) **and the sixty-first measured it** (DECISIONS 101
§1): nine lane gates past a lane's first against fifteen, five base
moves on row lists alone landed on their verdict with no re-gate, the
gate class 26.9 a session against 26.3 — **and waiting read 44.6 a
landing against 40.2, so the lane does not return for the
sixty-second**: the rise was the capture class, 3.2 → 9.8 a session on
five packets and run1419's two stalls, not the gate's, and a third
lane contends for both; the lane returns when a two-lane tranche reads
waiting under 40.2 and the gate class under 26.1 — the figure it was
taken at — or on Ramon's word; **the sixty-second read 42.3 waiting and
26.8 gate a session on two lanes** (DECISIONS 102 §1; the drain's handoff
had read 40.6 and 25.3 off an index that missed the last session's gate
wait), both over, **so two lanes for the sixty-third** — and where the
sixty-first's rise was the capture class, this one's is not: the capture
class fell 9.8 → 4.5 and the gate class held, because thirteen residue
items gated at 19.5–23.1 release minutes against the seven stand-ups'
7.8–9.3, the sim crate's own suite under `sim_crate_left`, which no lane
rule moves and Loop 3777 measures); **the sixty-third read 42.0 waiting
and 26.2 gate a session on two lanes** (DECISIONS 103 §1), both over by a
hair, **so two lanes for the sixty-fourth** — with no lane gate run twice
(six the tranche before) and the capture class at 2.4, the class is the
residue items' single gates, 20.4–23.5 release minutes against the
stand-ups' 8.3–9.8 — **and Loop 3777 measured those minutes as 273
emulator sweeps re-run past the cache cap on every gate, and the pass
built the compressed cache** (DECISIONS 103 §1): the five binaries 683.5
→ 41.8 s warm, so the sixty-fourth's gates run without them and its
drain reads the rule) — **and the sixty-fourth read 26.3 waiting and 15.3
gate a session on two lanes** (DECISIONS 104 §1), both under the figures
the lane was taken at and not by a hair: the residue lane gates 9.9–12.2
release minutes against 20.4–23.5, the stand-ups' 9.2–12.1, **so the
third lane returns for the sixty-fifth**, three Opus lanes at medium, read
at its drain against 26.3 waiting — a fifth above, 31.6, takes it away
again — and 15.3 gate, and watched on what the lane cost before: the door
(1,320.5 s on two lanes) and the lane gates past a lane's first (three
this tranche, none on a row list: 3804 on a moved base, 3810 twice, 3811
on an empty filter, Loop 3818)) — **and the sixty-fifth read 43.0 waiting
and 21.3 gate a session on three lanes** (DECISIONS 105 §1), 43.0 against
the 31.6 a fifth above 26.3 allows, **so the rule's letter takes the third
lane away for the sixty-sixth**, two Opus lanes at medium. What it cost
was the door and the chains, not the lane gates: admission 6,234.8 s
summed against 1,320.5, seven gate ends of fifteen past the cores against
one, the whole booking gates 50.4–53.2 release minutes against 44.1–49.4,
two merges of the two map chains conflicting in `floors.rs` (3825, 3828;
`message` 0.8 → 6.4 a session) and one lane 96 minutes on a hung mutant
(3831, Loop 3839) — while the landing read 6.21 USD against 8.82 and the
twenty took 9 h 04 against 11 h 39. The lane returns when a two-lane
tranche reads waiting under 26.3 and the gate under 15.3, the two-lane
figures it was returned on, or on Ramon's word, which the landing's price
and the tranche's clock argue for (DECISIONS 105 §1) — **and the
sixty-sixth read 25.0 waiting and 12.3 gate a session on two lanes**
(DECISIONS 106 §1), both under, **so the third lane returns for the
sixty-seventh**, three Opus lanes at medium, read at its drain against
26.3 waiting — a fifth above, 31.6, takes it away — and 15.3 gate, and
watched on the door (1,669.2 s on two lanes against 6,234.8 on the last
three) and on the whole gates' length (46.3–48.8 release minutes) —
**and the sixty-seventh read 27.3 waiting and 13.3 gate a session on
three lanes** (DECISIONS 107 §1): over the 26.3 it was returned on, under
the 31.6 that takes it away, and 21.4 without one lane's 93-minute
unguarded `multiprocessing` respawn (3886), the gate under 15.3, **so
three lanes stand for the sixty-eighth**, read at its drain against the
same two lines; the door 2,982.5 s and the whole gates 45.1–54.7 —
**and the sixty-eighth read 18.9 waiting and 7.5 gate a session on
three lanes** (DECISIONS 108 §1; the drain's "14.1 a gate" was the
booking gates' mean length, never the rule's figure, which is
`tranche.py`'s gate class a session), both under, **so three lanes
stand for the sixty-ninth**, read at its drain against the same two
lines, the door 540.0 s and the whole gates 48.5–51.1; **a tranche that
drains for want of items is read at its count** — thirteen, two slots
idle from landing 10 — and its supply is the pass's to fill, never the
rule's to answer: the implicit residue (108 §2) is the pool — **and the
sixty-ninth read 42.1 waiting and 15.2 gate a session on three lanes**
(DECISIONS 109 §1; the drain's 42.8 and 15.1 were read on 24 sessions,
the pass's on 25): the gate under 15.3, the waiting past the 31.6 that
takes the lane, **so the rule's letter takes the third lane away for the
seventieth**, two Opus lanes at medium. What it cost was the chains, not
the door: eight merges backed out on row-list conflicts (3909 with
3910, 3915 with 3916, 3920 with 3923 and once more, 3924) and re-landed,
each a lane gate of 12–46 minutes and a message wait — three chapter
lanes landing on the same four row lists conflict pairwise, a
three-lane cost by construction — and one clear held forty minutes by
its own `blocked` row (Loop 3931, built); admission 3,419.3 s and the
whole gates 46.3–52.6 held. The lane returns when a two-lane tranche
reads waiting under 26.3 and the gate under 15.3, or on Ramon's word;
the seventieth's supply is three items and what they park (109 §2), so
the third lane's absence costs one wave — **and the seventieth read
26.1 waiting and 10.6 gate a session on two lanes** (DECISIONS 110 §2;
seven landings, drained on supply): both under, **so the letter returns
the third lane, and the count follows the supply** — three Opus lanes
when the next tranche's first wave is three items or more, two
otherwise, and the supply is Ramon's, never the rule's to answer —
**and the seventy-first read 52.0 waiting and 21.0 gate a session on
three lanes** (DECISIONS 112 §1; thirteen landings, drained on supply):
the waiting past the 31.6 that takes the lane and the gate past 15.3,
**so the rule's letter takes the third lane away for the
seventy-second**, two Opus lanes at medium. What it cost was the chains
again, by construction: ten chain takes of one track landing in the
same two files (`floors.rs`'s `MapgenStyle` rows and
`rondata/src/mapgen.rs`'s arms) — 3950's merge backed out once and
3951's three times, each a `ccc update --keep-conflicts`, a lane gate
and a message wait (3951: 114.0 minutes waiting, 44.3 gate, 31.1
suite, 31.7 message) — and the row-list lever could not spare them,
since every one of those conflicts also touched code (`mapgen.rs`,
`map_lakes.rs`, `sim/src/lib.rs`), so no row-list regex is added; the
door 5,594.7 s and eight gate ends of twelve past the cores, the whole
gates 45.9–64.4 release minutes. The lane returns when a two-lane
tranche reads waiting under 26.3 and the gate under 15.3, or on
Ramon's word — **and the seventy-second read 54.9 waiting and 21.8
gate a session on two lanes** (DECISIONS 113 §1; fifteen landings,
drained on supply): both over, **so two lanes stand for the
seventy-third**. What it cost was the instrument track by
construction: nine of thirteen lane gates widened — every coverage item
moves a pin — and ran 31–53 release minutes (3963 52.9, 3964 49.6,
3970 39.3, 3971 38.0), the `other` class 11.9 a session that
`tranche.py` cannot name (3965 82.7; Loop 3996), one capture class of
65.5 (3961), the door 6,386.2 s and eight gate ends of fifteen past the
cores; the seventy-third's AI track is seven items that each move a pin
too, so its gate class is read for what the shape costs, not the lane
count. The lane returns on the same two lines or on Ramon's word; a
lane with no word of its own is a sweep lane, which runs the
original's functions under the emulator and contends for no capture
(DECISIONS 61) — **and with no battery word open and the sweep pool
spent, an Opus lane takes a `reading` or a `coverage` item** (entries 84
and 87; never Sonnet) **and a Sonnet lane the sweep rows the pass names, or stands
empty** (the forty-seventh pass, DECISIONS 86: the held-out battery ran
eight of eight whole, every battery word closed, the `port` and `name`
marks drained). **The commander counts landings since the last steering pass — by
`git log` from that pass's commit, never from its own last clear — and
stops at twenty**, writing the handoff and saying the steering pass is
due — **its `For the pass` counts are the tables'**: `tools/gates.py <tmp>
--since <the last pass's gate log>`'s last two lines and `tools/tranche.py
--base <base>`'s last, pasted, never a count from memory (Loop 3564: one
drain said "no red booking gate" over two, the next "one `DEEP`" over
three), **after `lore index`** (the sixty-second's drain read 40.6
waiting and 25.3 gate off an index that missed the last session's gate
wait; the pass read 42.3 and 26.8, DECISIONS 102 §1) — **or sooner, on the user's word, as a pit stop** (Loop 3439, entry
88: the forty-ninth drained at sixteen): **a capture that is running is
never stopped halfway** — a reprioritisation stops *future* captures
only, because a game cut short spends its run number and its lane for
nothing — **and a question that wants a cold look waits for the pass or
calls it early**, never a worker's mid-item redirect; **and a finding it would raise at that pass is filed in
`docs/PARKED.md`'s Loop section in the turn it is noticed**, the
handoff's `Fable backlog:` count moving with it — a finding narrated in
chat has told nobody (parked 509), **and a journal's own "for the Loop"
line is filed at its merge**: four journals of one tranche named the
same broken waiter and the pass found it in the table (656's sequel);
**the commander clears at the seam after every tenth landing** (`ccc
clear <own ref> --then continue`), the handoff written first, never in
the middle of a chain — a clear left free was not taken in three
tranches (`docs/DECISIONS.md` entry 55) — **and the arm is the turn's
last act, and the turn ends**: it fires only on an idle row, and a
commander that armed it and went on to merge the next landing
cancelled its own clear three times in one tranche and ran to 402 k
(parked 1314). **A landing that re-invokes the commander under an
armed clear is merged and the turn ends without cancelling**: the arm
stays through that turn and fires at its end; `ccc clear <ref>
--status` is the only question asked of it — never `--cancel`, and
`--status` reads "not armed" for some seconds while a fire is in
flight, which is not "dropped" (parked 1392; ccc 0.1.38). A landing
that arrives while the clear fires reaches the cleared session as the
turn after its prompt, measured. A second lane may run a
parked value-diff row *beside* the word's frame, never instead of it — a
rule about what it works on, never about when it is spawned.
`docs/DECISIONS.md` entries 40 and 45.

## The handover between the commander and the pass (retired)

**The commander and the pass spawn each other, and neither waits on a
person** (`docs/DECISIONS.md` entry 61). Each is a session of its own in
the integration worktree, started fresh: the pass ends by clearing the
commander's row into the next tranche — `ccc clear commander --then
"<the queue's opener>"`, armed with the tree committed — and the
commander, at twenty landings, with its handoff written, its gate green
and its lanes drained, ends by clearing the pass's row the same way:
`ccc clear steer --then "<the opener>"`. **That clear is the drain's last
act and the only one; a drain that stops without it is a stall** (Loop
3536: the fifty-second tranche's commander halted at twenty reading
"never message the steer row" as "never clear it", and the handoff waited
36 minutes for a person). **The steer's row is idle when the drain arms,
so the fire is immediate, and the drain reads it**: a backgrounded `sleep
150; ccc clear steer --status` is armed after the clear and the turn ends;
when it re-invokes the session, a `did not fire` — a person's unsent draft
in the steer's prompt box, which ccc never submits or deletes — goes to the
phone as a permission-shaped block, with the opener in the message (Loop
3821: the sixty-fourth's arm was refused that way and the pass began eight
hours late, on Ramon's question). The opener the pass writes names the verb. **The two rows are permanent
and `spawn --replace` is not used**: a cross-session clear fires (ccc
measured it, 2026-10-07, `docs/DECISIONS.md` entry 64 (iv)) when the
target's tree is committed at the arm and at the fire, the target is at
`tempo=idle`, and ccc.app or a headless attach is serving; the session
uuid changes and the name, model and bridge id stay. A stray row is
dropped with `ccc forget <ref>` (ccc 0.1.39), which keeps the worktree
and branch and refuses a row that still has a process (`ccc stop`
first) or a tree the harness or ccc cut — those are `ccc rm`'s (parked
1541: four stale rows stood for a day and the twenty-seventh pass
removed them). One of the two is
live at a time and only the live one edits the worktree; the `attrition`
row is the user's, where a steer with the user happens, and it edits
nothing there while a commander's lanes are out. **The two rows hand over by `ccc clear <row> --then
"<opener>"` and by nothing else; a clear is not a message.** Lanes
message the commander, as they must; **nobody messages the steer row**,
and the steer messages nobody — its product is the committed tree and
the opener it arms, and a decision taken with Ramon while a commander
runs is Ramon's to send or waits for the next pass (Loop 3499: the one
time it happened, a relayed decision came back hours later as a status
line that re-read a 300 k-token context; Loop 3536: "never message" was
once read as "never clear", and a drain stopped).
