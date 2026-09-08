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

## No rows outstanding

The five rows this file opened with — DECISIONS 37 as drafted, the perf/UB
ground item 280 opened, item 283's `#[global_allocator]`, the commander's
amendment of `CLAUDE.md`'s wait rule, and items 292/293 with lore in the
room — were ruled on 2026-09-07 by `2026-09-07-fable-pass-2.md`, which
keeps each row's text alongside its verdict. The next marked row goes here,
under a `## FABLE:` heading, with the evidence it wants the pass to weigh.
