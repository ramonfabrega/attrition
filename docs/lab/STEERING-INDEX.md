# Lab steering index

Status checked against open GitHub PRs on 2026-09-22. This is a navigation aid,
not a request to interrupt the main loop. Fable chooses what to adopt, pilot,
or park; none of these branches is a whole-merge recommendation.

| Checkpoint | What to review | Current lab view |
| --- | --- | --- |
| [PR #5](https://github.com/ramonfabrega/attrition/pull/5), draft | Native-verified retained-call replay and limit counterfactual | Retain as counterfactual evidence; not a whole-frame state oracle. |
| [PR #6](https://github.com/ramonfabrega/attrition/pull/6), draft | Native-verified saved-search graph comparison | Frozen checkpoint; continuation parks at L87, with L88 handoff on its existing branch. |
| [PR #7](https://github.com/ramonfabrega/attrition/pull/7), draft | [Typed-state acquisition and scalar bridge](TYPED-STATE-REVIEW.md) | Pilot. Same-frame scalar agreement and exact height bits are useful; whole-frame parity, liveness and initial height reconstruction remain unproven. |
| `codex/typed-leader-bridge`, follows #7 | [Leader arrays](2026-09-22-leader-bridge.md) and [world grids](2026-09-22-world-grid-bridge.md) and [complete observed GUY records](2026-09-22-guy-state-bridge.md) | Pilot broad scalar comparison: 516,369 matching occurrences / 179,583 distinct storage references; 164/226 branch-pinned unread keys covered on this frame. No new acquisition; whole-frame parity remains open. |

PR #8 was also open when checked, but belongs to the separate recomp spike;
this index neither reviews nor speaks for that work. Older Jev exploration is
summarized in [JEV-REVIEW.md](JEV-REVIEW.md); the chronological findings and
continuation handoff remain in [LEDGER.md](LEDGER.md).

For eventual steering, the concrete choice is whether to pilot the bounded
snapshot collector alongside existing capture instruments. Replacing the
logger is not supported by current evidence. Shared trace-hook adoption should
be reviewed separately from offline analysis tools. No score or main queue
change is proposed.
