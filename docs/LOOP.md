# The loop

The fan-out rules: the roster, the brief, the commander's chain, the lanes,
the ratification passes and the steer's fence. Moved whole from `CLAUDE.md`
by the forty-fourth pass (DECISIONS 83): every worker inherited these 38 KB
and none of them needed more than the landing paragraph, which `CLAUDE.md`
keeps. **Read by the commander and the pass; a worker reads `CLAUDE.md`
and its brief.** Amended in place, as `CLAUDE.md` was; a clause a commander
may land is the same one-clause safety fix, filed as a `FABLE:` row the same
day.

## The roster, and who spawns whom

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
from the fortieth, DECISIONS 78–82; `ccc spawn --model 'claude-opus-5-5[1m]' --effort medium` and
`--model 'claude-sonnet-5-5[1m]' --effort medium`, after three Opus
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

## The brief

**A brief is composed by `tools/brief.py`**: the frame
(`tools/brief/frame.md`), the item in the queue's own words, what is
reserved, and the commander's note of what the item alone needs — never
the previous brief, copied and edited. The frame and the checklist
behind it (`docs/audit/README.md`, "The brief checklist") are the pass's
to write, and a test holds every row of the one to a section of the
other; a worker's "for the Loop" line that names a brief's omission is a
candidate row. **The fourth kind is `reading`** (entry 84), beside
`residue`, `chapter` and `sweep`: a lane that reads the never-entered
functions its document cites into claims that each name their
falsifying capture, and gates on `python3 tools/release_gate.py
~/code/fun/attrition/game --test-threads 4 --lane --item <N> --tests
blind:: --no-widen`. **The fifth is `coverage`** (entry 87): a lane
that retakes one pinned lobby at `cover=1` **on the click-free pool**
(entry 88: `cover=1` runs there since 3429, and a click is an `@click`
line since 3437, so the queue lane is only for what no `@` line sends —
a key until `@key` lands, the menu's own code), checks the
trace against its `cover=0` twin — `samegame.py --from` the first
unshared input, where the take sends one — and pins it into
`blind::TRACES`, so the functions a reading's captures-owed table names
leave `NEVER`; it gates on the same line.

## The worker's landing

**A worker's landing is verified by refs and announced by the worker.**
Before reporting, a worker states its tip SHA and that `git log
<base>..<branch>` is non-empty — or, for a capture or a reading, where the
product is — and then sends its commander a one-line done message: a
report that only sits in the worker's own transcript has told nobody, and
a session's state is not a signal. **A worker commits before it gates**
— the gate's verdict is a second commit or an amend — so a lane that dies
mid-gate has its work on its branch and not on its floor; a gate's
notification was lost once and the work sat two days. **A gate's exit
is read before anything says pushed**: the gate is a background task
whose exit ends the wait, so a closing message written while it runs
has not landed, and a wrapper that echoes the exit has hidden it
(parked 574). **A gate that is running is not stopped for a base that
moved**: the lane takes `ccc update` at the gate's exit, lands on that
verdict if the update is clean and says which base the gate ran on, and
gates again only if the update conflicted — four were stopped in one
tranche, on no rule (entry 56). **A conflicted update is taken with
`ccc update <ref> --keep-conflicts`** (exit 3, the markers left in the
worktree): the lane resolves, `git add`s and `git commit`s the merge
itself — its own `git merge` is what the classifier denies, and one
lane sat 86 minutes for want of this (parked 1315; ccc 0.1.38). **A message that
arrives during a gate says it is to be applied after it**, and the worker
holds its write-ups until the gate exits. **An instruction about what to
start is never one about what to stop**: "pause new lanes" is answered
by not spawning and "when is the capture lane free" with a time read
off the lane lock, and neither by a message to a running worker — a
question about a resource is not a request for it (parked 507, the
eleventh pass). **An action announced in a
closing message is performed in that turn or it has not happened.** A
worker that has not landed **ninety
minutes** after its spawn sends a one-line status instead — the item, the
step it is on, whether a gate is running — because two lanes on one box
stretch every lane's wall clock, and wall clock is the one signal a
commander cannot read. **A worker's story is its own file**,
`docs/journal/<date>-item-<N>.md`; it touches neither `docs/JOURNAL.md`
nor `docs/QUEUE.md`, and the numbers it reports are the commander's to
book and to write on the scoreboard line. **A number a worker reports is
measured on the tree after its last `ccc update`**, or the report says
which tree it was measured on — an attribution taken against a moving
base is unreliable in both directions. On the commander's side, **merge,
book, spawn, gate, push and reap are one chain**; a reap left for "before the next
spawn" is a reap that does not happen. `docs/DECISIONS.md` entry 34.

## The commander's chain

**The commander's chain is ccc's, and it is one line.** `ccc merge <ref>
--no-ff`, the booking commit, **the spawn that refills the lane, cut off
that commit** — `python3 tools/brief.py … > brief.md && [ -s brief.md ]
&& ccc spawn … - < brief.md`, because a brief the tool refused (exit 2,
nothing on stdout) fed to `ccc spawn -` makes a promptless `draft` row
that sits until a person looks (Loop 3434, entry 88) — the gate to a file — **`--scope <the tip the last whole
gate ran on> --test-threads 8` on a merge, and the whole gate, no
`--scope`, every third landing whose merge widened and at the drain**
(the thirty-fifth steer, DECISIONS 72: a scoped gate runs the timed
tests only when a merge since the last whole one can move a pin; the
forty-first pass, DECISIONS 79: a sweep or document landing in third
position drew the whole gate by position, twice in one tranche) — **`git push origin
worktree-replan-pdb`, read back by `git rev-parse HEAD
origin/worktree-replan-pdb`** — `ccc push <ref> --base` pushes the
row's *base*, which for the integration row is `main` (parked 1742),
and the bare verb once answered "nothing to push" with origin two
commits behind (parked 1492) —
`ccc rm <ref>`, and `git push origin --delete <the lane's branch>` — a
worker pushes its branch, `ccc rm` deletes only the worktree and the local
branch, and thirty-seven merged `origin/worktree-att-*` stood on 2026-09-25
(parked 727) — never a raw `git merge`, never a hand-rolled reap, **no
spawn and no `ccc update` between the merge and its booking commit** (the
tree is red in that window), and **nothing under `docs/` is edited while
a gate runs**: the handoff is rewritten before the gate, not during it.
**A reap that refuses is a lane with work on the floor**: a killed lane
looks exactly like one that never started, so a handoff never says a lane
produced nothing until `git status` in that lane has said so. **A brief
reserves what two lanes could take in silence** — the run number and
the section number when another lane is in the same document, **and a
reserved section is one of the document's stub pool** — `## N. Reserved
for the next item on the <track> track`, landed six deep at a tranche's
opening and refilled when two remain, assigned by number in the queue's
words and never by an edit to the document, because a stub appended
beside the one a lane is rewriting is an adjacent hunk and a conflict
every time (parked 1491, 1718, 1741: seven of twenty-four merges backed
out; `docs_guard` holds the pool at two) — **and
code is not fenced**: two lanes in one
module merge, or conflict out loud at `ccc update` or at the gate; the
lane that lands second takes the update before its gate; a brief says
where the other lane's word sits as information, no grant is asked or
given, and **a worker never hands its word over for where the code
sits** (`docs/DECISIONS.md` entry 55). **And a brief names the lane's
park heading** — one of the parked file's pool, `## Parked by the next
landing, slot <A> — reserved`, landed six deep at the file's end,
assigned by letter, renamed by the lane, held at two by `docs_guard` —
because every lane appended its heading at one tail and three merges of
one tranche backed out there (parked 1812, DECISIONS 67). **A pinned
constant and its comment are the worker's to re-pin; the queue's lines
are the commander's to write**, and a worker whose gate is red only on
those lines has done its half — **which its gate says, under `--lane`,
in a `Lane verdict:` line the landing quotes**; "red only on your
lines" is never the worker's own reading of a log (parked 969). The
re-pin is split on purpose: **the
word's delta in the constant's comment, the word's block in the
widening test, and each says which** (parked 513). `ccc spawn --json`'s answer is never filtered — the
id lore's lineage reads is not the one the commander reads. **A booking
commit is never amended once another lane is live**: a lane merges the
base it sees, and a rewritten base leaves that merge parented on a commit
that no longer exists; a red gate on the booking commit is a second
commit. **The spawn follows the booking commit, and the gate runs
beside the new worker**: a landing frees a lane, and the turn that books
it refills it with the queue's first unstarted item on that track,
before its own gate — a lane stood empty through every booking gate of a
tranche — **and only while the landings so far plus the lanes live are
under twenty**, so the tranche drains together and no lane runs alone
(the thirty-ninth tranche's twenty-first ran an hour by itself; Ramon,
DECISIONS 77). **And a spawn is followed by a wait on the lanes** —
`python3 tools/lanewait.py <every live lane>`, backgrounded, re-armed
at its every exit: it exits on a push to a lane's branch, on a row
that left `working` and stayed out, and on a hundred quiet minutes.
Three lanes of one tranche wrote their landing in their own transcript
with the message tool never loaded, and the commander slept three
hours and thirty-four minutes until a person asked (parked 1412); the
idle notice built against that fires at every backgrounded wait —
three notices for one idle, none a landing
(`docs/audit/2026-10-04-sequential-arm.md`). A done line is the
lane's, and the wait is what wakes a commander when the line does not
come — **and a `DRAFT <lane>` line on the waiter's stderr is a push
under a clear, never a landing** (Loop 3368: twice in one tranche a
lane's draft journal, pushed for its `DEEP` answer, exited the waiter
`LANDED`) — **so a done line that arrives while a wait runs stops the wait
(`TaskStop`) and re-arms it without that lane in the same turn**: all
sixteen of the twenty-ninth tranche's `LANDED` exits came after the
lane's own line, each a whole-context turn that learned nothing (parked
1695). **And a `DEEP <lane> <ctx>k` exit is answered in its turn**: one
line to the lane — journal as a draft, commit, end the turn — and `ccc
clear <lane> --then "continue item <N> from the journal"`, armed by the
commander, since a lane's own arm is refused (DECISIONS 61) and five
sessions over 300 k took none (parked 1637, 1638). **And a `STALLED
<lane> <min>` exit is answered in its turn**: one line to the lane —
the gate or capture it waited on has exited; read its log and land —
because a lost exit notice leaves a row `working` at `tempo=idle` with
nothing running, where no other exit can fire, and one sat 178 minutes
(parked 1871, DECISIONS 67). **And an `UNCLEARED <lane> <min>` exit is
answered in its turn**: one message to the lane carrying the clear's
`--then` prompt — twice in one tranche a cross-session clear fired and
its prompt never ran, and the new session sat on its `/clear` twenty
minutes until a message woke it (parked 2127, DECISIONS 69). A red booking gate is a second commit and one line to every
live lane to take `ccc update`; **a landing that arrives while a gate
runs is merged at the gate's exit**, never under it. The turn does not
end with a lane empty and the queue not, and it never asks whether to
continue — the queue's opener is the answer.
## The lanes

**Lanes are independent.** Lanes are throughput, not a pair — **one to
an open word, and the queue's handoff says how many**: each lands, is
chained and is refilled without reference to another's state, and the
only coupling between them is the merge and what the brief reserves.
**A pass adds one lane a tranche while a landing's waiting holds** —
`tools/tranche.py`'s waiting minutes a landing no more than a fifth
above the tranche before — **and takes one away when it does not**; a
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
due — **or sooner, on the user's word, as a pit stop** (Loop 3439, entry
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

## Ratification, and the passes

An Opus adjudication is acceptable under the marker discipline — append each
verdict as it is settled, and mark what cannot be settled `FABLE:` rather
than guess. **Ratification runs in batches, over what is marked**, rather
than gating every mechanic on its own pass: markers and code-changing
verdicts accumulate, and a pass takes the accrued set. How large a batch and
how often is deliberately not fixed yet; what is fixed is that a mechanic is
not blocked waiting for one, and that nothing marked is quietly dropped.

**A ratification pass is not a subagent: it is the session.** Its brief
is a **charter, not a checklist** — the verdicts and markers are the floor,
the mandate is what the earlier passes missed, and a pass fenced to the
floor can only ever agree with the framing that fenced it.

**The commander and the pass spawn each other, and neither waits on a
person** (`docs/DECISIONS.md` entry 61). Each is a session of its own in
the integration worktree, started fresh: the pass ends by clearing the
commander's row into the next tranche — `ccc clear commander --then
"<the queue's opener>"`, armed with the tree committed — and the
commander, at twenty landings, with its handoff written, its gate green
and its lanes drained, ends by clearing the pass's row the same way:
`ccc clear steer --then "<the opener>"`. **The two rows are permanent
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
nothing there while a commander's lanes are out.

**An unattended pass has a fence.** It measures the tranche, ratifies the
marked rows, rules the Loop items, builds the loop's tools and guards,
and rewrites the queue and the loop's own clauses in this file — one
commit a person can revert. **Its first act is the archives' compression,
backgrounded** — `afsctool -c -f -T LZFSE -J8` on `~/ron-data` and
`~/ron-golden`, six minutes and 19 GiB on 2026-10-17, never beside a
gate (DECISIONS 77) — **and its table of the tranche's gates is
`python3 tools/gates.py <the commander's job tmp> --since <the last
pass's gate log>`** (DECISIONS 80). **It never moves what is the user's**: the
roster, the lane count past the one-a-tranche rule, the phase, the finish
line, the thesis, a hard constraint, or anything a decision entry
reserves. Those it writes as recommendations at the top of its record,
under "For Ramon", in twelve lines or fewer, with the tranche's landings
a day, its list price and what the pass changed, and it sends one push
notification saying the record is there. **Three things reach the user at
once, from the commander or the pass**: a kill rule tripped, a
permission-shaped block, and a booking gate still red after its second
commit. The user's silence is consent to nothing on the fenced list.

Launch a fan-out in waves, not whole; readers write to durable storage from
their first finding; verify which model actually ran from the transcript,
never from the spawn parameter. Rationale in `docs/DECISIONS.md` entries 22
and 23; the operating checklist in `docs/audit/README.md`.
