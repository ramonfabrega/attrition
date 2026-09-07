# The next Fable pass's agenda — rows marked, not yet ratified

`grep -rn 'FABLE:' docs/` finds this file and the queue's handoff line. It is
**the accrual**, not an audit: the batched ratification takes what is marked
here, and a mechanic never waits on it (`CLAUDE.md`, "Ratification runs in
batches, over what is marked"). Opened 2026-09-07 because there was nowhere
for a marked row to sit between passes — markers had only ever lived inside a
completed audit's own file, so anything marked outside one was findable only
by remembering it.

A row leaves this file when the pass rules on it, and the ruling goes in that
pass's own file under `docs/audit/`. Nothing is deleted from here silently.

## FABLE: `forbid(unsafe_code)` is tree-wide — DECISIONS 37, drafted

Written by the worker on item 280 and never ratified. The rule it states: the
ban covers the data reader as well as the sim, and a memory mapping is not a
permitted reason to narrow it. Worth a pass because the exception it reverses
was taken *in* `DATALAYER.md` and the journal and never reached `DECISIONS.md`
at all — which is how a tenet got narrowed without the user's word. The
question for the pass is not whether the mapping should have gone (it went,
with the user, and the tree greps clean) but whether the rule as written is
the right rule.

## FABLE: the perf/UB ground item 280 opened

The mapping was hand-rolled `unsafe extern "C"` mmap/munmap behind a **safe**
`read()` returning a `Deref<Target = str>`, with `from_utf8_unchecked` on top
— so a caller could not see the obligation, and a rewrite mid-read was UB
rather than a SIGBUS. It is gone. What is unratified is the general question
it raises: where else does this repo hide an obligation behind a safe
signature, and is `forbid` alone enough to catch that class. A pass with
perf/UB eyes is the right instrument; a diff is not.

## FABLE: item 283, the `#[global_allocator]`

Booked, not started. 280 measured the cost of dropping the mapping at
+1,563 MiB and established it is **not live data** — a freed `String` of a
capture's size is retained by macOS's allocator and cannot serve the next
capture's different size. An allocator that returns large blocks recovers most
of it with no `unsafe` in this tree. The pass should rule on whether adding a
`#[global_allocator]` is a dependency this workspace wants at all, since
`CLAUDE.md`'s rule is that a dependency is earned rather than anticipated.

## FABLE: `CLAUDE.md`'s wait rule, amended by the commander

Amended 2026-09-07 **by the commander with the user**, not by a steering
session, and flagged here because the fan-out rules reserve rewrites of that
file for Fable. The gap: the rule named the wrong way to wait (a foreground
sleep-and-grep loop) and stopped, so a worker obeying it to the letter had no
sanctioned way to be idle and invented one that was worse.

**The numbers moved twice and the second version is the one in the file.**
First report: `lane-286`, 107 no-op turns, "$28, 57% of the session". Both
figures were wrong — lore had priced Opus from memory at 3x the rate table,
and the share was computed against a shorter session. The true figures are
**$9.48 and 38%**. Worse, the same corrected scan found the shape was not
rare: a throwaway regex had matched a bare `echo` but not `echo .`, hiding
`loop-258`'s **282 idle turns in a run of 187, $39.99**, thirteen hours
earlier and unnoticed. Fleet-wide it is 447 turns over ten sessions, and
**five attrition worktrees are 93% of the bill**. So this is a shape of *this
repo's* workers — the ones that background a long external wait are exactly
the ones that reach for a way to stay alive.

The clause is "and then the turn ENDS", and the ban is deliberately on **any
command whose purpose is to yield the turn** rather than on a list: the first
draft listed `true`, `:` and `echo waiting`, and `echo .` — the costliest
case — would have walked straight through it.

Three things for the pass. The wording is the commander's and deserves a
second eye. The precedent is the real row: whether a one-line safety clause
with unambiguous evidence is a thing the commander may land in `CLAUDE.md`,
or whether every such fix waits for a pass while the gap stays open. And the
method lesson, which is the durable one: a canon rule carrying a measured
number is only as good as the measurement, and this one shipped wrong for an
hour — **a number in a rule wants its source named**, so the next reader can
check it rather than inherit it.

The generalization arrived from lore the same evening, and it is the sharper
statement: **the prose a tool ships is code, and wants the same gates code
gets.** Its corrected description then failed to compile — an unescaped
apostrophe closed a string in `cli.ts` — and 251 tests stayed green, because
the verb descriptions are imported by nothing. The syntax error reached
master and surfaced only when the installer tried to bundle it, with the bad
commit already pushed. The parallel here is exact and unguarded: `docs_guard`
reads the queue's *shape* — item count, section pins, handoff length, cited
function addresses against the decompile index — and nothing reads a **number**
in `CLAUDE.md` or a mechanic document. So the ledger on one paragraph of this
repo's canon is three failures deep: a figure recalled rather than measured,
a commander restating it, and the upstream correction that did not compile.
Whether that warrants a guard, and what such a guard could even assert, is a
question for the pass rather than a change the commander should invent.

## FABLE: is the loop paying for itself — items 292 and 293, and lore in the room

Added 2026-09-07 by the commander that ran the loop, which is the first thing
the pass should hold against these rows: the framing below is the framing that
fenced itself, and a pass that only rules on it can only agree with it.

**Ramon's own doubt is the row, and it was raised while two workers ran:** "i
dont think the current loop is working good, or maybe these 2 were very big
issues, but im pretty sure something is wrong" — after 289 and 290 had been
running about two hours. He named the options himself: optimise the pipeline
before deepening it, or fall back to a single runner, which he doubts. He also
named the instrument: **lore**, for archeology and stats on whether the guards
and the gate are improving the flow, rather than another anecdote. `lore tools`
counts how often `guard.sh` actually runs against the release gate, `lore
trace` and `lore usage` price the turns, and `lore polls` already prices the
waiting that grows around long runs. The two items are parked as **292**
(nothing in this tree has ever been profiled — no bench target, no criterion,
no flamegraph, grepped) and **293** (the pipeline question), in
`docs/PARKED.md` with the evidence.

**What was measured, and it is not what the suspicion assumed.** Neither
worker was stalled and neither was in a red-gate loop. 289's release gate went
green on its third run — 245 tests, peak 11,218 MiB — and its two red runs
were expected re-pins of numbers its own change had moved. The hour went to
**one unoptimised debug `cargo test`: 4101.85 s against 322 s for the same 245
tests in release, 12.7x**, measured. What the debug run buys over the release
gate is **eleven `debug_assert!`s** and no `#[cfg(debug_assertions)]` path at
all, because `[profile.release]` already sets `overflow-checks`. So
`[profile.dev] opt-level = 2` would keep all eleven and cost the incremental
compile speed `tools/guard.sh`'s reflex is built on.

**Two corrections came from the workers themselves and both cut against the
easy conclusion.** 290: a third of that contention was its own debug run
launched **twice by mistake**, so this pair cannot indict entry 34's width
two, and an operator error and a structural cost look identical from outside.
What clean width two costs is **unmeasured**. 289: the cost is not only
throughput — two lanes on one box inflate every lane's **wall clock**, and
wall clock is what a commander reads to decide whether a lane is stuck. That
is exactly the misreading that made this commander go and ask both workers
whether they were cycling on a red gate. **The loop's own health signal
degrades as width rises**, which is a defect in the discretion entry 34
grants, not in any lane.

Three things for the pass, and only the first is on the floor. Whether the
opt-level trade is worth taking, which is bounded — it costs `guard.sh` and
nothing else. Whether width two survives contact with a health signal it
degrades, or wants a rule rather than discretion. And the one this commander
cannot ask honestly: **whether an unattended commander loop should be running
at this width at all while its own throughput is unmeasured** — the stopping
rule counts landings that move no score, and has nothing to say about a
tranche that lands everything slowly.

**What this tranche actually produced**, so the pass has the other side: two
items in about two hours, both landed and merged, East Indies 7806 → 7812, and
290 refuted the premise it was booked on — the original holds 88 food on 7585,
our own number, so 287's `98 <= food < 160` was wrong and every future worker
sent after that stockpile would have been wasted. A capture that refutes its
own hypothesis is the loop working, not failing, and the pass should weigh it
against the wall clock rather than only the wall clock.
