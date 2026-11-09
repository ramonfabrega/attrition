# The loop

The loop's machinery: the driver, the roster, the brief, the worker's
landing, the chain, and the passes. **Read by the driver; a worker reads
`CLAUDE.md` and its brief.** Amended in place. The seventy-sixth pass
(2026-11-09, `docs/DECISIONS.md` entry 116) made one driver of the commander
and the steer and moved the clauses that retired with them, whole, to
`docs/audit/2026-11-09-loop-retired-clauses.md`. **"The commander" in a
clause below is the driver.**

## The driver

**One session drives, and it is the session Ramon talks to.** The `steer`
row, on Fable 5.2, is the only session that owns the integration worktree.
It books, merges, gates and pushes; it holds the direction of the track it
is driving; it spawns the workers; and at a boundary it is the pass. The
`commander` row is retired. Two permanent rows alternating on one worktree
was a hazard by construction — a row not cleared, a row woken, both live —
and their handover was the most scarred text in this file.

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
queue's opener>"` as the turn's last act, and the turn ends. An arm fires
only on an idle row, so nothing follows it; it is asked only with
`--status`, never cancelled; and a draft left in the prompt box, which ccc
never submits, is a permission-shaped block that goes to Ramon in the
closing message. **It clears at a milestone of the track it is driving, at
a supply boundary, and whenever its next turn would carry more than 230 k**
— the depth at which landings were measured to get dear (lore, 2026-11-09:
lane USD a landing followed the count of deep sessions, 5.03 in a tranche
with none and 10.61 in one with twelve). Idle compaction is the fallback,
never the mechanism: a session that resumes summarised re-reads the queue
before it acts.

**Supply calls the pass, not a count.** The driver's next session opens as
a pass when the queue holds fewer open items than the workers it would
feed, when the track's next step is undesigned, at a milestone, or on
Ramon's word; twenty landings without one of those is also a boundary, as
a backstop. The pass is a fresh context by construction, and the clear
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
  fifteen-minute gate is untested (Loop 4053). lore does not index these
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

- **The driver**: Fable 5.2.
- **The chain** — start, checksum, orders and save on one lobby, to the
  milestone `docs/TRACKS.md` names — is the driver's own, built by
  in-session builders whose model is the driver's call for each step: Opus
  5.5 for a mechanical build, Fable 5.2 where the step is design. This is
  the pilot. It is not a controlled trial: it is judged by the chain's
  milestones landing with every floor held, and by points of the weekly
  window a milestone.
- **The render track**: one ccc lane, Opus 5.5 at `--effort medium`, under
  the frame. It is the control: 27 tranches of that roster read 7.25 USD a
  landing (lore, 2026-11-09).
- Never Sonnet on a reading or an adjudication.

**Two kill rules ride with the pilot**: a floor that falls with no named
regression and is pushed; and no chain milestone inside ten points of the
weekly window. On either the chain goes to an Opus lane under the frame,
and Ramon is told at once. No driver changes the roster or the worker
count: it scores them and recommends under "For Ramon".

**Three things reach Ramon at once**: a kill rule tripped, a
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
  session. The waiter's `DRAFT`, `HELD` and `UNCLEARED` clauses in "the
  chain" were written against the old behaviour and are unverified (Loop
  4051).
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
candidate row. **The fourth kind is `reading`** (entry 84), beside
`residue`, `chapter` and `sweep`: a lane that reads the never-entered
functions its document cites into claims that each name their
falsifying capture, and gates on `python3 tools/release_gate.py
~/code/fun/attrition/game --test-threads 4 --lane --item <N> --tests
blind:: --no-widen`. **The fifth is `coverage`** (entry 87): a lane
that retakes one pinned lobby at `cover=1` **on the click-free pool**
(entry 88: `cover=1` runs there since 3429, and a click is an `@click`
line since 3437), checks the
trace against its `cover=0` twin — `samegame.py --from` the first
unshared input, where the take sends one — and pins it into
`blind::TRACES`, so the functions a reading's captures-owed table names
leave `NEVER`; it gates on the same line. **A need for the queue lane
again is a steer, never a lane's redirect** (item 3485, DECISIONS 89).
**A late-age pair is a `residue` item that stands a word up** (DECISIONS
90): the Mongol pair's shape — its own start and long at `cover=0`,
`end:UNITS=3,GUYS=2` on a stride, the age held and read back,
`diff::<name>_pair` pinned at its first parting — scored by its first
try, its partings residue items on that word. **The rubric reads lockstep
games only**: the worker runs `tools/rubric.py` on its tree once the
word is pinned, names the types new to the ledger in its journal and
re-pins `census::RUBRIC_*`, and the commander writes the queue's
`Rubric:` line at the booking as it writes `Blind:` — the pass's
`--record` row stays the pass's.

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

Written for the commander; the driver runs it for every ccc lane. Its
waiter clauses are to be re-measured on the new harness (Loop 4051) and
then shortened (Loop 4055).


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
`ccc rm <ref>`, and `git push origin --delete <the lane's branch>` —
**the push reads the gate log's own `gate exit 0` line first, never
the background task's exit** (Loop 3997: the seventy-second's drain
pushed a tip whose whole gate was red on `diff::audit::`, its check
keyed on the harness's exit line; a red tip on origin is a red tip the
next spawn inherits) — a
worker pushes its branch, `ccc rm` deletes only the worktree and the local
branch, and thirty-seven merged `origin/worktree-att-*` stood on 2026-09-25
(parked 727); **the classifier may refuse that delete** (`[Git
Destructive]`, from the commander's row and the steer's alike, Loop 3848):
a refusal is reported at the drain with the count, never retried — never a
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
sits** (`docs/DECISIONS.md` entry 55). **And a brief names the lane's
park heading** — one of the parked file's pool, `## Parked by the next
landing, slot <A> — reserved`, landed six deep at the file's end,
assigned by letter, renamed by the lane, held at two by `docs_guard` —
because every lane appended its heading at one tail and three merges of
one tranche backed out there (parked 1812, DECISIONS 67). **The letters
are two capitals and cycle**: `ZZ` is followed by `AA` (the sixty-seventh
tranche's commander wrapped there, DECISIONS 107 §4), and a pair is free
when no heading in the file carries it and no open item assigns it — a
landed heading is renamed to its item, so a pair comes round free; the
guard reads two capitals and never a third. **A pinned
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
line to the lane — journal as a draft (its head says `draft`, or a
`Left to do` heading stands anywhere in it: the waiter reads both, Loop
3925), commit, push, end the turn, **or,
if the item is finished, land instead** (Loop 3833: four of thirteen
`DEEP` lanes over three tranches were past their status and pushed the
landing, every one with its lane gate still running) — **and the draft
turn ends done, on "this session is done", never `blocked` and never
with a reply suggested** (Loop 3931, the sixty-ninth pass: lane-3919's
armed clear stood forty minutes on its own `blocked` row and suggested
reply, which ccc reads as a person's unsent draft and never submits;
the waiter exits `HELD <lane> blocked` with that remedy, where it said
`ENDED`) — and
the waiter re-armed with `--arm-after-push <lane>`, which arms `ccc
clear <lane> --then "continue item <N> from the journal"` at that lane's
next push and prints `ARMED` or `ARM-REFUSED` — **a push on a lane whose
own gate still runs is `GATING … held, not armed`**, read again at the
gate's exit, and a push whose journal is no draft is `PUSHED … a
landing, not armed` (Loop 3833 built; the sixty-fifth pass) — **and
`LANDED` needs the journal's `Lane verdict:` quote or the row's done
line; a push with neither is `PUSHED … progress, not a landing: its
journal quotes no lane gate yet`**, and a landing whose newest commit is
not the quote's exits `ENDED` with the row's line (Loop 3853 built; the
sixty-sixth pass: a fresh session's re-pin push read `LANDED` with no
gate run) (Loop 3526: `ccc clear`
refuses a lane with uncommitted work, so an arm in the answer's own
turn failed twice and the commander's line had already said "armed");
a lane's own arm is refused (DECISIONS 61) and five sessions over 300 k
took none (parked 1637, 1638); **a `HELD <lane> <range>` line before
any exit is a draft push still unarmed** — the re-armed wait with
`--arm-after-push` arms it at start when the row is idle and unarmed
(Loop 3765: two held drafts died with a wait that exited on the other
lane); **and a `GATING <lane>` note is a row that left `working` with
its lane gate still running, held and not an exit** (Loop 3753: 3747's
`ENDED` within a minute of `WAITING`, twice). **And a `STALLED
<lane> <min>` exit is answered in its turn**: one line to the lane —
the gate or capture it waited on has exited; read its log and land —
because a lost exit notice leaves a row `working` at `tempo=idle` with
nothing running, where no other exit can fire, and one sat 178 minutes
(parked 1871, DECISIONS 67) — **unless the lane holds a draft push,
when the stall is the arm**: the waiter arms the clear itself, prints
`ARMED`, keeps watching, and nothing is sent (Loop 3795: 3788's held
draft sat through two `HELD` exits and a `STALLED` with the row reading
`working`, and was armed by hand). **A message is never sent to a lane
beside an armed clear**: the clear fires on the row's idle and the
message lands in the fresh session as its second turn, which obeys it —
3788's fresh session was told "end your turn" and did, until a
correction ten minutes later (Loop 3796). A nudge and an arm are two
answers to one stall; the arm is the one that does not need the lane
to read anything. **And an `UNCLEARED <lane> <min>` exit is
answered in its turn**: one message to the lane carrying the clear's
`--then` prompt — twice in one tranche a cross-session clear fired and
its prompt never ran, and the new session sat on its `/clear` twenty
minutes until a message woke it (parked 2127, DECISIONS 69). A red booking gate is a second commit and one line to every
live lane to take `ccc update`; **a landing that arrives while a gate
runs is merged at the gate's exit**, never under it. The turn does not
end with a lane empty and the queue not, and it never asks whether to

## The lanes

**The worker count is Ramon's** (DECISIONS 114 and 116): one ccc lane on
the render track and the driver's builders on the chain. A pass reports the
shape of the waiting and recommends; it does not move the count. The rule
that added and removed a third lane on waiting minutes is retired, with its
history, to `docs/audit/2026-11-09-loop-retired-clauses.md`: it moved the
lane five times in eight tranches without naming what it waited on, and
what it waited on was the gate.

**Lanes are independent**: each lands, is chained and is refilled without
reference to another's state, and the only coupling is the merge and what
the brief reserves. **A capture that is running is never stopped halfway**:
a reprioritisation stops future captures only. **A finding the driver would
raise at a pass is filed in `docs/PARKED.md`'s Loop section in the turn it
is noticed**, the handoff's `Fable backlog:` count moving with it, and a
journal's own "for the Loop" line is filed at its merge. **The handoff's
`For the pass` counts are the tables'** — `tools/gates.py <tmp> --since
<the last pass's gate log>` and `tools/tranche.py --base <base>`, pasted
after `lore index`, never a count from memory. **A waiter on a lane is
armed with `--deep 230`.**

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
marked rows, rules the Loop items, builds the loop's tools and guards,
and rewrites the queue and the loop's own clauses in this file — one
commit a person can revert. **Its subagents' worktrees are removed at
their merge** (`git worktree remove`, before the pass's gate: the
fifty-second pass's, locked by a live `claude bg-spare`, refused the
fifty-third tranche's tenth booking gate until a person removed it, Loop
3545) **and the lane rows a drain left idle are reaped at the pass's
start** (`ccc rm <ref>` on a merged row; eight stood at the fifty-third's
drain, Loop 3565). **Its first act is the archives' compression,
backgrounded** — `afsctool -c -f -T LZFSE -J8` on `~/ron-data` and
`~/ron-golden`, six minutes and 19 GiB on 2026-10-17, never beside a
gate (DECISIONS 77) — **and the integration tree's debug profile is
cleaned beside it** (`cargo clean --profile dev`: 180,851 files and 74.2
GiB on 2026-11-04, the disk 108 → 144; the reflex rebuilds it in minutes)
**when `df -g ~` reads under the last pass's figure** (DECISIONS 104 §6;
the sixty-fifth read 114 at its start, 125 after both; the sixty-sixth
141, over 125, so the archives alone, 147 after; the sixty-seventh 138,
under 147, so both: 48,997 files and 24.9 GiB cleaned, 151 after)
— **and its table of the tranche's gates is
`python3 tools/gates.py <the commander's job tmp> --since <the last
pass's gate log>`** (DECISIONS 80), **and its rubric is `python3
tools/rubric.py --record "<the pass>"`** — the content view (ages held,
unit types seen, source groups wholly backed) as a row in
`docs/audit/rubric.tsv` and a table against the pass before, in the
record beside the clock; the queue's `Rubric:` line is the pass's to
re-pin and never a booking's (Ramon, the forty-ninth steer, DECISIONS 88). **And its blind list's open rows are scanned for references** —
`tools/trace/report.py <exe> refs -` over `blind::NEVER` net of
`RESIDUE` — **and the dead ratified by family, each row with the scan's
reason**, because 115 of 192 open rows were out-of-line copies no
capture could enter and three tranches had booked captures against the
list without knowing it (Loop 3465, the fiftieth pass, DECISIONS 89); a
residue row is a claim no capture can break, and one the console's
`!eval` or an `@` line can reach is struck, never restated on scope
(Loop 3473). **It never moves what is the user's**: the
roster, the worker count, the phase, the finish
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
