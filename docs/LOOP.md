# The loop

The loop's machinery: the driver, the roster, the brief, the worker's
landing, the chain, and the passes. **Read by the driver; a worker reads
`CLAUDE.md` and its brief.** Amended in place. The seventy-sixth pass
(2026-11-09, `docs/DECISIONS.md` entry 116) made one driver of the commander
and the steer and moved the clauses that retired with them, whole, to
`docs/audit/2026-11-09-loop-retired-clauses.md`. **"The commander" in a
clause below is the driver.**

## The driver

**One session drives at a time.** Since 2026-11-12 (DECISIONS 126) the
driver is the `commander` row on Opus 5.5 and the pass is the `steer` row on
Fable 5.2, alternating on the integration worktree. The row that is live
books, merges, gates and pushes, holds the direction of the track it is
driving and spawns the workers; "the driver" below is that row. Entry 116
had merged the two rows because both had been live at once — a row not
cleared, a row woken. What holds them apart now: **exactly one row exists
in the tree, and a handover is a spawn and a reap** (DECISIONS 130, 131 §8;
Ramon, 2026-11-13). The row that hands over banks, writes the other's
opener to a file and runs `python3 tools/handover.py to <commander|steer>
--opener-file <file>` as its last act: it refuses an unpushed tree and a
background command still running under the row, reaps any other driver
row, and spawns the new one with the opener as its prompt. The new row's
first act is `python3 tools/handover.py reap <the old ref>`: stop, then
forget, never `ccc rm`, which reaches the tree. **No prompt is typed into
a pane and no row rests on its context.** Both ways of doing it otherwise
failed: a resting row was woken cold by its own background command an
hour later (312,347 tokens written, 2026-11-12), and ccc's clear types no
prompt onto a row that is already empty (the pass armed at 01:30 on
2026-11-13 never started). `tools/gate_run.sh` asks `handover.py check`
before every gate and refuses, exit 95, while two driver rows hold the
tree. `steer` writes in the tree only on a turn a handover opened.

**Ramon steers at will, in this row.** A message he types here arrives
inside the running turn and is answered in it. A decision of his is written
to `docs/DECISIONS.md` before the next clear, because a clear forgets the
conversation. There is no second row to wake and no rule against talking
while workers are out.

**The driver banks, then clears itself.** Banked is all of: the tree
committed, the gate log's own `gate exit 0` read, the branch pushed and
`origin/main` fast-forwarded, the queue's handoff rewritten, and no
in-session builder in flight. A ccc lane may be out across a clear: the
handoff names it, its item and its waiter's arguments. Then the driver
reads `ccc clear <own ref> --status`, arms `ccc clear <own ref> --then "<the
queue's opener>"` as the turn's last act, and the turn ends (its own row
has a screen to clear; the other row is never cleared, only spawned). An arm fires
only on an idle row, so nothing follows it — **and the arming turn's closing
text asks nothing**: an ask leaves the row `blocked`, and an arm sat seven
hours on one (DECISIONS 118); what is Ramon's goes in the handoff. It is
asked only with `--status`, never cancelled; and a draft left in the prompt box, which ccc
never submits, is a permission-shaped block that goes to Ramon in the
closing message. **It clears at a milestone of the track it is driving, at
a supply boundary, and whenever its next turn would carry more than 400 k**
(230 before DECISIONS 131 §6: measured on four lanes cleared at 230 k, the
fresh session's re-reading cost more than the lighter requests saved, 7.77
USD against 4.27, because a token read again costs forty times a token
carried; the figure of 2026-11-09, 5.03 USD a landing with no deep session
and 10.61 with twelve, followed the item's size). **What is dear is a read
past the hour the cache lives**: a row or a lane that will sit longer than
that on a deep context is cleared first. Idle compaction is the fallback,
never the mechanism: a session that resumes summarised re-reads the queue
before it acts.

**Supply calls the pass, not a count.** `commander` hands over to `steer` when the
queue holds fewer designed items than the workers it would feed, when the
track's next step is undesigned, at a milestone, when a finding changes
the direction or the kill rule trips, or on Ramon's word; twenty landings
without one of those is also a boundary, as a backstop (DECISIONS 126 §4). The pass is a fresh context by construction, and the clear
before it is what makes its scoring a second look. What a pass does is
under "Ratification, and the passes".

**Who builds.** The driver holds the design and does not build in the
integration tree, where a half-done step would sit under every merge and
every gate. A build goes to a worker in a worktree of its own, and the
driver chooses the shape for each step:

- **An in-session builder** — the Agent tool with `isolation: worktree`.
  Its brief is the goal, the score it moves, what is ruled out and where to
  read, with the machine facts; never the frame. It builds red first, runs
  `tools/guard.sh` and its own tests, and hands back its branch and tip.
  The driver merges it and gates once, at the merge; on a red gate it sends
  the builder the failing test, and the builder fixes in its own context. A
  builder never runs the release gate: whether one waits cleanly on a
  fifteen-minute gate is untested. lore does not index these
  spawns, so their price is the weekly window's delta, read before and
  after.
- **A ccc lane** — `ccc spawn --name lane-<N> --worktree=att-<N> --model
  <m> --effort <e>`, briefed by `tools/brief.py`, gating itself with the
  lane gate and landing by the worker's contract below. **`--worktree` on
  every spawn**: without it a lane runs in the integration checkout (Loop
  3837). lore sees a lane as a job, and Ramon can attach to it.

The model of each worker is said in user-visible text at its spawn, and
verified from its transcript, never from the spawn line. A builder on
another model takes that model's own default effort, not the driver's.

**The roster is Ramon's, set at a steer** (DECISIONS 116):

- **The driver**: the `commander` row, Opus 5.5. **The pass**: the `steer`
  row, Fable 5.2, spawned at a boundary (DECISIONS 126, 131 §8; entry 122's
  observer, given the tree for the length of a pass).
- **The chain** — start, checksum, orders and save on one lobby, to the
  milestone `docs/TRACKS.md` names — is the driver's own, built by
  in-session builders whose model is the driver's call for each step: Opus
  5.5 for a mechanical build, Fable 5.2 where the step is design. This is
  the pilot. It is not a controlled trial: it is judged by the chain's
  milestones landing with every floor held, and by points of the weekly
  window a milestone.
- **The lanes**: three ccc lanes, Opus 5.5 at `--effort high`, under the
  frame — four workers with the chain's builder, confirmed at the pass by
  load (DECISIONS 129 §1). **At most one lane holds an item the queue marks
  whole**; a scoped item that finds it must touch `crates/sim` says so
  before it builds, **and in its done line, and its gate stays the brief's**
  (DECISIONS 132 §6: no lane gates whole but a `structure` lane; two that
  did sat 18 and 21 minutes on the host-wide lock before a twenty-minute
  run, and the booking gate ran the same tests again). A `--lane --whole`
  gate still takes that lock and prints `Whole lane: waiting on <holder>`. The trial
  tranche ran four of nine lane landings whole and read two false reds
  under a load of 46 to 51 on 16 cores. One lane at medium was the control: 27
  tranches of that roster read 7.25 USD a landing (lore, 2026-11-09); the
  nine lane landings after the reset read 8.36.
- Never Sonnet on a reading or an adjudication.

**One kill rule rides with the pilot, and a soft stop with the week**
(DECISIONS 121, which removed the window's kill rule). The rule: a floor
that falls with no named regression and is pushed — on it the chain goes
to an Opus lane under the frame, and Ramon is told at once. **The stop:
99 % of the weekly window since 2026-11-13 (95 % from 2026-11-11, 85 % before), Ramon's to move
at will by a word on this row.** At or past it the driver cuts no builder and refills no lane; what
is in flight finishes and is booked; the driver banks and ends its turn
with a report. Inside three points of it nothing is cut that cannot be
booked before it. The wall is never met mid-work: it kills every agent in
flight, and the resume is the dear part. The handoff reports the window's
reading, the stop, the pace against the reset and what each step cost. No
driver changes the roster or the worker count: it scores them and
recommends under "For Ramon".

**Three things reach Ramon at once**: the kill rule tripped, a
permission-shaped block, and a gate still red after its second commit.

**`origin/main` is the trunk, and it is always a gated tip.** The driver's
branch is `worktree-replan-pdb` for history's sake. After every green gate
it pushes that branch and fast-forwards main — `git push origin
worktree-replan-pdb:main`, a fast-forward only — because a harness-made
worktree branches from `origin/main`, and a main left behind starts a
builder on a stale base. The root checkout is Ramon's to pull. Renaming the
branch is his: a remote branch delete is refused from every row.

**The harness moved under this file** (Claude Code 2.1.300 to 2.1.321, read
on 2026-11-09). What changed, and what it makes of a clause below:

- **A clear no longer kills a background agent's running commands.** A
  gate under an armed clear keeps running and its exit reaches the cleared
  session; what the waiter's exits mean now is `tools/lanewait.py`'s
  header, which is the authority.
- **A message to a running background agent arrives at once.** A line to a
  lane whose gate is running now wakes the lane mid-gate, at a full context
  read. It is sent only when the lane must act before its gate exits.
- **A subagent on another model takes that model's own effort.**
- **Idle compaction** summarises a resting session over 200 k. The driver
  clears before it rests, so it should not apply.
- **`fable` is Fable 5.2** on every row that named it.

## The brief

**A brief is composed by `tools/brief.py`**: the frame
(`tools/brief/frame.md`), the item in the queue's own words, what is
reserved, and the commander's note of what the item alone needs — never
the previous brief, copied and edited. The frame and the checklist
behind it (`docs/audit/README.md`, "The brief checklist") are the pass's
to write, and a test holds every row of the one to a section of the
other; a worker's "for the Loop" line that names a brief's omission is a
candidate row the pass reads in the journals (there is no Loop backlog:
DECISIONS 125 §3). **The kinds and their gates are `tools/brief.py`'s
header**; the ones no item is booked under now (`reading`, `coverage`, the
late-age pair's shape, the rubric's re-pin) are retired to
`docs/audit/2026-11-09-loop-retired-clauses.md` ("Retired 2026-11-14: brief
kinds no item is booked under"). **`structure` and `render`** (Loop 4022)
**and `start` and `input`** (DECISIONS 128 §7) **are the sixth to ninth**:
a start lane gates on its own filters since DECISIONS 132 §6 (`--lane
--whole` before it), an input lane scoped with the capture section. **A
need for the queue lane again is a steer, never a lane's redirect** (item
3485, DECISIONS 89).

**A builder's brief is not composed from the frame.** The driver writes it:
the goal, the score, what is ruled out, where to read, the machine facts,
red first, and what to hand back. The frame's rows are the method of a word
lane, and a chain step is not one.

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
lane sat 86 minutes for want of this (parked 1315; ccc 0.1.38). **A
`Base moved:` whose every shared path is a row list on the base's side
lands on the gate's verdict** (Loop 3694, the sixtieth pass): the
line names them — a row list's added rows, a document, a capture's
row, the census pin — and the lane updates with `--keep-conflicts`,
keeps both sides' rows, re-pins, runs `tools/guard.sh` and lands,
saying which base the gate ran on; the booking gate is the check on
every merge. A removed row or any other path is still an update and a
second gate. **A message that
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

**A word may fall on a named regression** (DECISIONS 104 §2; rule 864,
`docs/audit/README.md`): a landing whose value diff closes a standing row
on the widening, whose fall is named by its frame and cause with a probe
that reads above the old word, and whose new word books the next item,
lands with the floor re-pinned down and the queue's line reading "a named
regression from <old>" until the word passes it again — 3814 took the
third melee 755 → 718 that way. Holding the old word by leaving a right
arm out is the parity gate the thesis names; landing the probe would pin
what does not exist.

## The chain

Written for the commander; the driver runs it for every ccc lane. The
waiter's exits, and what answers each, are `tools/lanewait.py`'s header,
which is the authority (Loop 4051 and 4055 ruled closed by the pass of
2026-11-14).

**The booking gate takes every landing that is merged when it starts**
(DECISIONS 131 §4). The tranche of 2026-11-12 ran eleven booking gates one
landing at a time, 334.9 minutes of gate in a span of 413: the gate bound
the loop, not the workers. The driver merges and books every lane and
builder that is done, then gates the tip once and pushes it; a red names
its test, and the driver reads which merge moved it before it sends anyone
a line. **At a builder's merge the pins a merge moves are run first** — the
census pin, the doors pin, `free::ages`, `diff::coverage`,
`diff::checksum_lines`, `diff::transfer`, `diff::floors` — because two of
that tranche's three reds were such a pin found by a thirty-minute gate,
and the next tranche's first was a literal in `diff::floors` (gate-4205,
19.8 minutes on it alone; the literal is gone, DECISIONS 132 §4).

**A booking gate is scoped, and the whole gate runs at a drain and at a
pass** (DECISIONS 132 §5). A scoped gate widens to every timed test on any
path that can move a pin; what the whole gate adds is the caches' verifiers.
Of nine red verdicts in the 43 gate logs of 2026-11-11 to 11-13 none was a
test a scoped gate skips, and the verifiers caught nothing: 20.0 minutes
against 33.0, by the medians. **A lane's own gate runs its brief's filters,
the paperwork guards and whatever the timings file has never timed**
(§6): the rest is the booking gate's, on the merged tree. A red at the
booking gate names its test; the fix is its owner's second commit, and a
merge whose fix is not in hand is reverted (`git revert -m 1`) and lands
again with it.

**A tree's corpus is what its own ledger names** (DECISIONS 132 §3;
`rondata::corpus`). From run 1546 on, a capture in the shared archives
counts in a tree's tests only when that tree's `docs/RUNS.md` heads a
section with its exact run token, so a capture filed by one worker is
skipped and printed by every other tree until the filing branch merges
(run1575 and run1576 turned four gates red on 2026-11-13). **Each run
token takes its own heading or is named in one; a range is not expanded.**
A recording a capture leaves in `Logs\` carries no run token and is not
covered: a capture that plays a file removes the `.rcx` it left, or says so.

**The gate clears the disk** (DECISIONS 128 §1). After a gate that exits 0,
`tools/gate_run.sh` runs `tools/disk.py`: every oracle cache generation but
the gated build's **and those a process read within twelve hours** leaves
both archives (DECISIONS 129 §2: for a tranche the sweep removed the
generations in-flight workers were reading, 130 to 239 a gate, and they
rebuilt cold), and a builder's tree whose tip is
merged and whose tree is clean is removed with its branch. The log's
`Disk:` lines say what went and what is free, `LOW` under 30 GB; the driver
reads them beside `gate exit 0`, **and `Host top:` beside `Host load:`**:
the five commands with the most CPU at the gate's end and start, which is
how a tranche learns what loads the host past the budget's twenty slots.
**A timed test that fails "unmeasured, re-run" is the host, not the
client** (DECISIONS 129 §3): the gate is re-run when the load has fallen,
and nothing is diagnosed. The disk read 0 GB inside a builder's run
on 2026-11-12: about 94 generations stood in each of 133 caches and seven
merged trees held 40 GiB of build output, and nothing removed either. A
capture is never removed this way (entry 84 (iv)).

**The commander's chain is ccc's, and it is one line.** **A done line is
checked against origin before anything is merged** — `git ls-remote
origin refs/heads/<the lane's branch>` prints the tip the line names,
or the lane is told to push and the waiter's `LANDED` is waited for
(Loop 3503: three lanes of the fifty-first tranche sent their done line
with the tip on the local branch only, `ccc merge` read the local ref,
and a reaped worktree would have held the only copy) — then `ccc merge <ref>
--no-ff`, the booking commit, **the spawn that refills the lane, cut off
that commit** — `python3 tools/brief.py … > brief.md && [ -s brief.md ]
&& ccc spawn … - < brief.md`, because a brief the tool refused (exit 2,
nothing on stdout) fed to `ccc spawn -` makes a promptless `draft` row
that sits until a person looks (Loop 3434, entry 88) — the gate to a file — **`--scope <the tip the last whole
gate ran on> --test-threads 8` on a merge, and the whole gate, no
`--scope`, at the drain and at a pass** (DECISIONS 132 §5) — **`git push origin
worktree-replan-pdb`, read back by `git rev-parse HEAD
origin/worktree-replan-pdb`** — `ccc push <ref> --base` pushes the
row's *base*, which for the integration row is `main` (parked 1742),
and the bare verb once answered "nothing to push" with origin two
commits behind (parked 1492) —
`ccc rm <ref>` —
**the push reads the gate log's own `gate exit 0` line first, never
the background task's exit** (Loop 3997: the seventy-second's drain
pushed a tip whose whole gate was red on `diff::audit::`, its check
keyed on the harness's exit line; a red tip on origin is a red tip the
next spawn inherits) — a worker pushes its branch and `ccc rm` deletes
only the worktree and the local branch: **the merged branches on origin
are Ramon's to delete or to allow** (DECISIONS 129 §9; the classifier
refuses a session the delete, `[Git Destructive]`, Loop 3848), and the
pass's record carries their count for him — never a
raw `git merge`, never a hand-rolled reap, **no
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
sits** (`docs/DECISIONS.md` entry 55). **And a brief tells the lane
where a finding goes**: a `## Parked` row of its own journal file,
`- **<item><letter>. title**`, and one line under the mechanic's "What is
not established" when it is about one. There is no parked file and no
pool of headings (`docs/DECISIONS.md` entry 125): every lane appended to
that one file, three merges of one tranche backed out at its tail (parked
1812, DECISIONS 67), and by 2026-11-12 it held 1.49 MB. **A pinned
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
`python3 tools/lanewait.py <every live lane>`, backgrounded, re-armed at
its every exit. **Its header is the authority**: every exit (`LANDED`,
`ENDED`, `HELD`, `QUIET`, `DEEP`, `STALLED`, `UNCLEARED`) and every line
that is not one (`WAITING`, `PUSHED`, `DRAFT`, `ARMED`, `ARM-REFUSED`,
`GATING`), each with its answer and the case that taught it; the
clauses this file carried are retired to
`docs/audit/2026-11-09-loop-retired-clauses.md` ("Retired 2026-11-14: the
chain's waiter block"). Three rules the tool cannot hold: **an exit is
answered in its turn**; **a done line that arrives while a wait runs
stops the wait (`TaskStop`) and re-arms it without that lane in the same
turn** (parked 1695: all sixteen of the twenty-ninth tranche's `LANDED`
exits came after the lane's own line); and **a message is never sent to
a lane beside an armed clear** — the clear fires on the row's idle and
the message lands in the fresh session as its second turn, which obeys
it (Loop 3796). A red booking gate is a second commit and one line to every
live lane to take `ccc update`; **a landing that arrives while a gate
runs is merged at the gate's exit**, never under it. The turn does not
end with a lane empty and the queue not, and it never asks whether to
continue — the queue's opener is the answer.

## The lanes

**The worker count is Ramon's** (DECISIONS 114, 116 and 129 §1): the
roster above holds the count. A pass reports the
shape of the waiting and recommends; it does not move the count. The rule
that added and removed a third lane on waiting minutes is retired, with its
history, to `docs/audit/2026-11-09-loop-retired-clauses.md`: it moved the
lane five times in eight tranches without naming what it waited on, and
what it waited on was the gate.

**Lanes are independent**: each lands, is chained and is refilled without
reference to another's state, and the only coupling is the merge and what
the brief reserves. **A capture that is running is never stopped halfway**:
a reprioritisation stops future captures only. **A finding the driver would
raise at a pass is written in the turn it is noticed, in the journal file
of the item it came from or in the pass's own record under `docs/audit/`**;
a journal's own "for the Loop" line stays in the journal and is filed
nowhere (`docs/DECISIONS.md` entry 125). **A handoff banked for a pass
carries a `For the pass` part, and its counts are the tables'** —
`tools/gates.py <tmp> --since <the last pass's gate log>` and `tools/tranche.py --base <base>`, pasted
after `lore index`, never a count from memory. **A waiter on a lane is
armed with `--deep 400`** (DECISIONS 131 §6).

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

**An unattended pass has a fence.** It measures the tranche, ratifies the
marked rows, rules what the tranche's journals and its own record raise,
builds the loop's tools and guards, and rewrites the queue and the loop's
own clauses in this file — one commit a person can revert. Its first acts (the figures earlier passes
read are in `docs/audit/2026-11-09-loop-retired-clauses.md`, "Retired
2026-11-14: the pass's first acts, the figures"):

1. **The archives' compression, backgrounded** — `afsctool -c -f -T LZFSE
   -J8` on `~/ron-data` and `~/ron-golden`, never beside a gate
   (DECISIONS 77).
2. **The suite's re-timing beside it** — `python3 tools/suite_timings.py
   <install> --stale-only`, committed (DECISIONS 132 §6: on 2026-11-13
   the file was four days old and 189 tests were untimed, so every lane
   gate ran them).
3. **The integration tree's debug profile cleaned beside it, when `df -g
   ~` reads under the last pass's figure** — `cargo clean --profile dev`;
   the reflex rebuilds it in minutes (DECISIONS 104 §6).
4. **The lane rows a drain left idle reaped at the pass's start** —
   `ccc rm <ref>` on a merged row (eight stood at the fifty-third's
   drain, Loop 3565).
5. **Its subagents' worktrees removed at their merge** — `git worktree
   remove`, before the pass's gate (the fifty-second pass's, locked by a
   live `claude bg-spare`, refused the fifty-third tranche's tenth
   booking gate until a person removed it, Loop 3545).
6. **Its table of the tranche's gates** — `python3 tools/gates.py <the
   commander's job tmp> --since <the last pass's gate log>` (DECISIONS
   80).
7. **Its rubric** — `python3 tools/rubric.py --record "<the pass>"`: the
   content view (ages held, unit types seen, source groups wholly backed)
   as a row in `docs/audit/rubric.tsv` and a table against the pass
   before, in the record beside the clock. The `--record` row is the
   pass's and never a booking's (Ramon, the forty-ninth steer, DECISIONS
   88); the queue's `Rubric:` line is written at a booking from the pins
   the worker re-pinned (DECISIONS 90).
8. **Its blind list's open rows scanned for references** —
   `tools/trace/report.py <exe> refs -` over `blind::NEVER` net of
   `RESIDUE` — **and the dead ratified by family, each row with the
   scan's reason**, because 115 of 192 open rows were out-of-line copies
   no capture could enter and three tranches had booked captures against
   the list without knowing it (Loop 3465, the fiftieth pass, DECISIONS
   89); a residue row is a claim no capture can break, and one the
   console's `!eval` or an `@` line can reach is struck, never restated
   on scope (Loop 3473).

**It never moves what is the user's**: the
roster, the worker count, the phase, the finish
line, the thesis, a hard constraint, or anything a decision entry
reserves. Those it writes as recommendations at the top of its record,
under "For Ramon", in twelve lines or fewer, with the tranche's landings
a day, its list price and what the pass changed, and it sends one push
notification saying the record is there. **Three things reach the user at
once, from the commander or the pass**: the kill rule tripped, a
permission-shaped block, and a booking gate still red after its second
commit. The user's silence is consent to nothing on the fenced list.

**A pass that designs a render step looks first**: it draws the client's
still at each photograph's camera (`client --frame N --still`) and sets it
beside the photograph before it reads a score. The pass of 2026-11-12
found washed figures, untextured trees and a mountain a third as bright
under rows that read whole (DECISIONS 127).

Launch a fan-out in waves, not whole; readers write to durable storage from
their first finding; verify which model actually ran from the transcript,
never from the spawn parameter. Rationale in `docs/DECISIONS.md` entries 22
and 23; the operating checklist in `docs/audit/README.md`.
