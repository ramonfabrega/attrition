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
