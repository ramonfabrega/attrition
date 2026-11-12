# The method

How the long middle was run: the rules for booking an item against a score,
for pinning a word with its widening, for captures, for readings, and for
waiting. **Moved whole from `CLAUDE.md` by the seventy-sixth pass,
2026-11-09** (`docs/DECISIONS.md` entry 116): every worker and every
subagent inherited these clauses, and a render, start or client lane uses
almost none of them. `CLAUDE.md` keeps each rule in a sentence; this file
keeps the clause as it stood, with the case that taught it.

**Who reads it.** A `residue`, `chapter`, `coverage`, `reading` or `sweep`
item reads this file before its first probe, and its brief says so. The
driver reads it when it books one. Where this file and `CLAUDE.md` differ,
`CLAUDE.md` is the rule and this is the detail. Amended in place, as
`CLAUDE.md` was. "The commander" in a clause below is the driver
(`docs/LOOP.md`, "The driver").

## Phase 3's scoring and succession

As `CLAUDE.md` stated phase 3 while it was open. Every word it names is
closed and stands as a floor.

3. **Sim skeleton** — the rules of a match, mechanic by mechanic, diffed
   against the original's own per-frame dumps. **The score is ticks before
   divergence** on each of the two maps the finish line names — the lower
   map first, and the default item is its nearest divergence — pinned as
   floors in `rondata::diff` and stated first in `docs/QUEUE.md`. **Done when** a
   fixed-seed, traced human-versus-AI capture on two maps stays in lockstep
   — no position or order disagreement — for its full length. This is the
   long middle, and the number is how anyone can tell where in it we are.
   Closing the current captures is the milestone, not the phase:
   `docs/DECISIONS.md` entry 29 names the three counters that define the
   sim complete, and the renderer waits on them. **A pair that closes is
   succeeded by the one whose lobby reaches the most census rows no
   trace has entered** — it names those rows before its capture is
   booked, and it may vary more than one setting — and the AI's word
   is the newest pair's; a closed pair's pins stay as floors
   (`docs/DECISIONS.md` entry 53, amended by 61). **A third map is
   scored in the first pair's lobby beside them, and the held-out map
   is measured and never debugged against** (entry 54) — **and so is
   a held-out battery, at every pass**: games nobody tuned for, each
   scored by how far it runs on its first try, which is the number
   that says when minting pairs can stop (entry 61). **A game that ends closes
   with its closing state scored, and the lower map is the lower open
   word** (entry 55). **A closed map is succeeded by the same map in
   the newest pair's lobby, and the rules track stages the arms the AI
   track built and no walk holds** (entry 56).

## Booking an item

**An item is booked with the score it moves.** The headline, or a sub-score
the queue names — and the item nearest the headline's first divergence is
the default. **A finding that names no score parks in its item's journal**
(`docs/DECISIONS.md` entry 125) rather than queues: the queue holds only what names the headline's frame,
a floor, or a takes-chain to one. **And the headline slot is never
empty**: when no open item names the current word's frame and draw, the
next brief is to widen every dumped record on that frame and name the
cause, not to run the nearest residue row. An item that turns out to be
something else writes its successors in its own journal, and a session
that moved no score says so in the handoff. **A word that moved lands with the
value diff beside it** — the dump's own coordinates on the frame it moved —
because a draw stream can agree on a wrong destination for a long time,
and only a value comparison tells the two apart — **and a widening's row
count is not one**: the value diff names the record, the field and both
sides' values on the word's frame — or, where nothing dumped parts
there, on the frame the state first parted, walked back from it — and
eight of one tranche's twenty landings reported a floor triple in its
place. This is the stopping rule
for the long middle: the residue chase is productive and unbounded, and
only the number says whether the whole is converging. **A residue item is
booked by its frame and its draw delta**; a mechanism its title names is
the previous item's hypothesis and is written as one, because the frame
has been right every time the named mechanism was wrong
(`docs/DECISIONS.md` entry 42). **And a hypothesis the floor kills is
not built by the item that killed it** — it parks with its kill; one
was built anyway "because it was the booked hypothesis", and the
booking is a frame, never a promise to build a name.

## The definition of done, with the gate line in full

**Definition of done**, all five:

- `docs/<MECHANIC>.md`, stating how it was established, how confident it is,
  what it has *not* established, and which of its claims a diff backs.
- The implementation, in its own module.
- Tests, including the end-to-end kind that run the new mechanic against the
  ones already there.
- `cargo test --release`, `cargo clippy --all-targets` and `cargo fmt`
  clean, and `cargo run -p rondata -- <install>` exiting zero — which is
  `python3 tools/release_gate.py <install> --test-threads 4` in one
  command, under the memory cap, with the fixture audit and the paperwork
  guards; that is the gate a landing names. **A worker's is the same
  line with `--lane --tests <the word's filters>`** — the brief names
  the filters and adds every lobby's `first_parting` (parked 1910, 2000:
  a fix on one map broke another lobby's floor and took a capture to
  read); the lane gate runs those, the fast tests and the guards,
  and leaves the timed ones to the booking gate (item 1567: 703 s → 74
  s) — red only on the queue's lines — the commander's to write at the
  merge — or on the journal's own gate quote, which cannot exist before
  the gate has run (3277: fourteen first gates of one tranche were red
  on it alone) — it goes on to the guards and exits 0, and red on anything
  else it stops and names the test; the commander's booking
  gate takes no flag and forgives nothing. The debug profile is for the
  reflex (`tools/guard.sh`) and for stepping through one test; a debug
  run that reaches a kept dump refuses, by design.
- Committed. Any open question this closes in another document is struck
  through there and pointed at its answer, per the amend-in-place rule below.

## The behavioural check

**A behavioural check is a logged run, and it is cheap.** The original runs
here (`docs/ORACLE.md`, last section): fix the seed in `rise.ini`, enable the
mechanic's categories under `[End Frame]` in `gamelog.ini`, play a minute, quit
through the in-game menu, read `Logs\gamelog.txt`. A claim that needs a
behavioural check is still written into the document's open questions with the
check named — and then, when it is the cheapest way to settle it, the check is
run rather than deferred. Do not enable everything per frame; it slows the
simulation to a crawl.

## Diffs before readings

**Prefer a diff to a reading, and convert readings into diffs.** Reading is
how we find out what to check; a differential check against the original's
own dump is how we *know*, and it keeps knowing on every later commit. Two
rules follow:

- **When the original dumps a record, diff the whole record** — every slot,
  every field — not the field the mechanic happens to care about. Nine tenths
  of a dumped record once went uncompared for a month, and the first widening
  failed on its first run. **And every unit on the frame, not the ones the
  brief names**: a call chain names a function, never a cast, and the whole
  cast either side of a frame answers "who changed" without a hypothesis.
  **A row is a field, never a unit** — a key that collapses a record's
  fields hides all but the first for the rest of the run — and the blocks a
  residue count gates on the position are read again at the parting,
  because the frame a position parts is the frame a word is read on.
  Twice in a day the named mechanism was wrong and the widening said so in
  twenty minutes.
- **A word is pinned with its widening.** The first item on a new word
  compares every dumped record on the word's own frame, both directions,
  and the pin names that test (`rondata::diff::testkit::WIDENINGS`) or
  the item that owes it; no item that names a mechanism runs on a word
  whose widening is not on file. A value diff read a few frames past the
  word is not the widening, and one once stood in three documents for
  four items. **A widening's own framing is a hypothesis** too: a stanza
  writes what would kill each reading before the run, **and a killer
  tests the claim's own unit, never the first row** — one was false and
  its killer did not fire (item 711). **A payoff probe
  changes only the frames under test** — one wide enough to touch a
  frame that already agrees reports that frame's breakage as its result.
  **A probe of a decision the original makes mid-frame runs inside the
  decision**, never at the frame boundary: the two are different
  measurements, and the cheap one has named the wrong mechanism.
  **A dumped record's slot index is not an identity**; compare on what
  the slot holds.
- **A finding that can become an assertion must become one before its audit
  is closed.** A second reading's budget is best spent on *what to assert*,
  not on more prose; the twenty-minute widening has out-produced the
  million-token reading.
- **Diff first, then read what no run reaches.** Before a blind reading,
  widen every dumped record the mechanic touches; then brief the readers
  with the mechanic's *never-executed* functions, not the whole mechanic,
  and ask each claim to name the capture that would falsify it — the ini
  category, the frame, the field. A claim a run has already confirmed does
  not need a second reader. The soak tests the simulation against itself —
  determinism and termination, never fidelity — and stays for that.
- **Grep the dump before booking a reading.** An open question whose
  answer is a field the original already prints costs a `grep` and has
  more than once cost a reading instead.
- **Grep this crate for a field before reading the original's writers of
  it.** A comparison against a field this crate does not carry is a
  comparison against nothing, and one once carried five items.
- **And read what the tree says it left out, by name, before the
  original.** `tools/seams.py` prints the live `SEAM`s and the "not
  modelled" paragraphs that name a word's chain; `tools/standing.py`
  puts a widening's standing keys beside who in `sim` reads them. Both
  were rows of the checklist in prose, and the landings that filed them
  had been briefed with them (`docs/DECISIONS.md` entry 56).
- **A quiet field is checked against the reader before it is called
  agreeing.** A key the dump prints that nothing parses cannot part, and
  quiet is what a blind instrument looks like; `rondata::diff::coverage`
  pins every such key on the headline windows, a reader added to the
  harness is added to its driver, and a key a landing reads leaves the
  pin in the same landing. **And a word's widening puts its window in
  the coverage driver**: the pin walks only the windows it is handed, so
  a record a new capture prints stays unpinned until its window is
  driven (621). **And the compared pin walks the newest word's
  window**: a field pinned as uncompared on a closed pair's window is
  a field nobody watches on the open one (parked 1067).
- **And grep the disk before booking a capture.** Widen every dumped
  record the mechanic touches first; book the capture only for what no
  record already on disk can answer, and **the booking cites what the
  disk could not answer**. The same rule one level up, and it
  orders the *booking*, not the screen — an idle screen may still run
  the capture lane. **A field is cited for a question only when its
  write condition can answer it**: a footprint written on one branch
  is silent on the other (parked 508).
- **Where a reading's product is a formula, the implementation is a pass
  of the audit — so build before ratifying, or alongside.** Prose can cite
  every address correctly and still have the arithmetic wrong, and an
  adjudicator cannot tell without doing the work: a row read as
  "additions, as cited" is exactly the row nobody re-derived. Writing the
  code, and running the diff, is what finds that. So a mechanic whose
  reading yields arithmetic gets its implementation *before* the
  ratification pass, or in parallel with the reading; a mechanic whose
  reading yields predicates and call graphs does not have to wait.

The last one is a **default, not a gate** — the point is that tools and
implementations earn their place ahead of reads wherever they can, not
that a fixed order is owed.

The reason reading does not go away is coverage, and it is measurable: the
trace's report (`tools/trace/report.py … blind docs/`) lists the functions
`docs/` cites that have never executed in any traced game. A diff can only
check what a run reaches, so for those the reading is the only evidence there
is. **Every behavioural run shrinks that list, and shrinking it is what would
eventually make the second reading unnecessary** — so the list is the queue
of runs, and a run that lights up a whole family is worth more than one that
lights up a function.

## Conventions as they stood

The host admission clause and the waiting clause, with their measurements.

- **Earn every dependency.** Crates appear in this workspace when they have
  real code, not in anticipation. Same for third-party deps.
- **A probe shape reached for a third time graduates into `tools/`.** The
  recurring dump probes — a slicer, a draw tracker, an entity scanner —
  are authored once and invoked thereafter; one-off hypothesis probes
  stay scratch scripts, because a wrong abstraction costs more than the
  typing it saves.
- **Host admission stays** (`docs/DECISIONS.md` entries 75 and 76: the
  thirty-eighth tranche waited 32.0 minutes a landing against 36.5 with
  the admission waiting 2–41 s a gate): `tools/guard.sh` and every gate
  mode reserve from one host budget (`tools/resource_budget.py`), each
  gate's last lines carry `Gate minutes:` by step, an `Admission wait:`
  receipt and `Host load:` (DECISIONS 80), and `touch
  ~/Library/Caches/attrition/admission-off` is the kill switch for every
  worktree at once. A standalone `cargo test …` joins the budget as `zsh
  tools/check.sh test …`; nothing wraps `guard.sh` or a gate again. The
  design and its limits are `docs/TEST_PERFORMANCE.md`.
- **`tools/guard.sh [filter …]` is the reflex between edits** — the
  paperwork guards plus the current item's tests, debug, seconds. The
  full `--release` diff suite remains the gate: before a commit and
  after a document rewrite.
- **A wait on an external process is backgrounded** (`run_in_background`,
  Monitor), never a foreground sleep-and-grep loop — every poll turn
  re-bills the whole context — **and then the turn ENDS.** The harness
  re-invokes the session when the task exits, so there is nothing to stay
  alive for: a no-op turn held open to wait gains no information and costs
  a full context read each time. **A detached capture is waited on with
  `tools/gamelog/waitrun.sh <viadriver log>`** in that background lane —
  it keys on the receipt lines the click-free runner prints and on a
  capture script's `=== captured: ` line, and a wait keyed on any
  other file has sat for two hours on a finished capture (parked 656)
  — **and with the Bash tool's `timeout: 3600000`, past the capture's
  length** — the tool's argument, never a `timeout` command, which
  macOS has not: the background default is thirty minutes, and a wait
  died at it before the banner (parked 1672); a wait prefixed with the
  command exited at once and reported a running capture as done (Loop
  2599).
  **The ban is on any command whose purpose
  is to yield the turn**, never on a list of spellings — `true`, `:`,
  `echo waiting` and `echo .` have all been used here, and enumerating them
  is how the next one gets through. `lore polls` measured the shape on
  2026-09-07 — the figures and their source are in
  `docs/audit/2026-09-07-fable-pass-2.md` — and every session that did it
  was one of this repo's own workers, which had backgrounded its wait
  correctly and been told only how *not* to wait. **A number in a rule
  names its source**: the first figure written here was wrong by 3x.
  **And a price names the day of its rate table**: a tranche once read
  half as dear as the one before and had cost the same (entry 56).
- **A multi-line Rust patch from Bash rides the python-heredoc pattern**
  (`python3 - <<'PYEOF'` with `old="""…"""`/`new="""…"""`), chained with
  its test run in the same call — edit and verify in one turn, and safer
  than `sed` across lines. **Never chain a `git checkout <file>` onto an
  edit that can fail**: the checkout runs when the edit does not, and it
  reverts the work already there.
- Toolchain is pinned in `rust-toolchain.toml` so the Solana toolchain on this
  machine can never leak in.
- Format recon notes live in `docs/FORMATS.md`; decisions and their rationale
  in `docs/DECISIONS.md`. A decision that gets overturned is amended in place
  with its successor named, never deleted.
- One document per mechanic, written from the original and implemented from the
  document — `docs/ATTRITION.md` is the first. Each states how confident it is
  and lists what it has not established, so a reader can tell a derived formula
  from a plausible guess.

## Tooling as it stood

- `llvm-pdbutil` (Homebrew LLVM) reads `rise.pdb` on macOS. `dump --types`
  works; `pretty` needs the Windows DIA SDK and does not.
- Ghidra 12.1.3 (`brew install ghidra` — a formula now, not a cask; it wants
  `openjdk@21`). With the PDB loaded it gives named, typed decompilation.
  **`tools/ghidra/` holds everything**: `analyze.sh` builds the project once
  (hours), `export.sh` decompiles all 48k functions plus every struct and
  vtable to files (minutes), and after that reading is `grep` over
  `decomp/` rather than a two-minute pass per question. `run.sh` runs the
  remaining one-off scripts. Its README lists the traps that have each cost a
  wrong conclusion once. The two things the decompiler does not do for you —
  name the field behind a `field_0xNN`, name the method behind
  `(*(code **)(*this + 0xcc))()` — are `types.txt` and `vtables.txt` in the
  export.
- **The original runs on this machine, on nothing we do not control.**
  Free WineHQ Stable 11.0 in the prefix `~/wine-ron`, with DXVK-macOS for
  the one D3D11 device the game asks for. `tools/gamelog/prefix.sh` builds
  the prefix, `winelaunch.sh` is the single launch line every capture script
  sources, and `focus.sh` the single window query; see `docs/ORACLE.md`,
  "Off CrossOver", for why each is needed and `docs/DECISIONS.md` 32 for why
  it is not a licence. `cliclick` drives it; System Events clicks do not
  reach it. The launch line holds a **lane lock** keyed on the capture
  script's pid and the game's: a capture takes it before its first
  staging write, a second launch into a held lane refuses and names the
  holder, and the lock releases itself when both have exited. It carries
  each pid's start time, so a pid handed to another process holds
  nothing; `ron_lane_state` says which of free, stale and held the lane
  is, and a stale lock is never removed by hand.
- **A function of the executable can be called outside the game.**
  `tools/emu/callfn.py` maps it under unicorn (`uv run`, dependency declared
  in the script) and enters a function with chosen arguments; a sweep is a
  `#[test]` in `crates/sim`. Free for a pure function, an hour of synthesized
  state for one that reads a singleton; `docs/EMULATOR.md` has the costs.
- **And it can be called on the original's own state.** The lab's two
  rungs (`docs/DECISIONS.md` entry 47; `docs/EMULATOR.md`'s last section
  has the costs): a **packet** — `tools/explore/frame_snapshot.py`,
  a `RON_STATE_FRAME` tracer build — copies the process's private data
  at one frame boundary, and `tools/recomp/step4.py` runs a function on
  it under unicorn with its writes watched; `tools/recomp/lift.py` is
  the native twin for a pure function and the float oracle on chosen
  inputs. Reach for them when the question is **a value the original
  holds or computes** — a value row on a widening, a float residue, a
  field no dump prints — before a reading and before a detail capture;
  not for "why did it diverge", which is the draw stream's. **A packet
  is taken at the word's own frame, read as a logger frame**; it is a
  second game on the single capture lane, ~4 min and 843 MB; and lifted
  C is a reading tool exactly as the decompiler is — never a source,
  never in git.
- Constants are not all loaded in the representation the file writes. At least
  one rational arrives scaled to 8.8 fixed point. Read the consumer before
  believing the digits.
- `cargo run -p rondata -- <install>` surveys the data layer and re-derives
  every structural claim in `docs/FORMATS.md` from the user's own files. If a
  claim stops being true it exits non-zero. Run it after touching anything
  that reads the game's data.
- **The lab** (`docs/lab/`, `tools/explore/`, `tools/viewer/`) is a second
  harness's exploration, merged whole on 2026-09-17 (`docs/DECISIONS.md`
  entry 38). Its claim ledger is `docs/lab/LEDGER.md`; the guards do not
  scan it. Two of its products are opt-in tools: a read-only diff viewer
  with in-memory replay checkpoints (`docs/DEBUG_VIEWER.md`), and a
  click-free capture lane (`tools/explore/unattended_capture.py`, a
  `RON_AUTOSTART` tracer build) that needs no TCC grant and no human at
  the menu, and starts most pairs rather than all of them.
