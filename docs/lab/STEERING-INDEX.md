# Lab steering index

Consolidated for the steering pass on 2026-09-22. Adoption remains Fable's
choice; the lab is score-neutral. The A* line is **unreviewed, not rejected**: no
steering pass has yet ratified the new lab work. Closing its PRs only reduces
the active review surface; all branches, commits and evidence remain available
for adoption or a fresh proposal.

| Review surface | Disposition |
| --- | --- |
| [PR #7](https://github.com/ramonfabrega/attrition/pull/7) | The one live typed-state checkpoint, rebased on `origin/main` `d892281`, including the entire former PR #9 tranche. Read [TYPED-STATE-REVIEW.md](TYPED-STATE-REVIEW.md), starting with the two optional tracer hooks. Recommendation: pilot, not logger replacement. |
| [PR #9](https://github.com/ramonfabrega/attrition/pull/9) | Closed as folded into #7. Historical test/evidence refs remain in the ledger. |
| [PR #5](https://github.com/ramonfabrega/attrition/pull/5) and [PR #6](https://github.com/ramonfabrega/attrition/pull/6) | Closed as superseded review surfaces; **not rebased**. A* replay/continuation stays parked at L87, with L88's later observation retained. L64–L88 and their named branch refs preserve the findings and counterfactual capability. Guard `1eea6d0` belongs to the parked post-graph collector, not the typed-state pilot. |
| `codex/capture-reuse-exploration` | Deleted after verifying it has no commits absent from main. No remote branch existed. |

The recomp spike, [PR #8](https://github.com/ramonfabrega/attrition/pull/8), is
independent and untouched. It examines the function side of the market offer;
this lab examines the read-only packet's state side. Neither has a blanket
correctness claim over the other.

Coverage expansion is stopped. The remaining investigation for this pass is
charter step 4: item 327's Merchant slot, regional rare counters and wealth at
trace tick 11185 / logger frame 11186, bounded to roughly one hour. The [result](2026-09-22-market-state-head-to-head.md) answers the endpoint
state with 206 agreeing fields; the offer mechanism needs an earlier boundary.
