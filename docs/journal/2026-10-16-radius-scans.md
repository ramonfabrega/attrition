# Scoped live-unit/radius scans — 2026-10-16

User-authorized sequential performance follow-up on `codex/astra-windowed-reader`,
starting at `cb19d62ed98fe4dd9669d3692b7d62ea453fd8df`. Commander remains paused.
The goal remains 10× with less CPU and memory, especially under four/six-suite
contention. No test restructuring or dropped checks is proposed in this experiment.
Evidence root: `/tmp/attrition-radius-scans-20261016-01/`. The original release
binary is preserved as `baseline-rondata`.

## Selection and candidate

A fresh 20-second native sample of the original coverage walk (`profile.py`,
`profile.log`, `sample.txt`) confirms radius scans beneath `orders::spot_sweep`:
463 leaf samples in `find_unit_with_radius`; 197 in `wonders_held`. Both sample
and test exit0. Sampling localizes work; it is not a speed measurement.

A placement sweep borrows Sim immutably. The candidate `collide::RadiusScan`
keeps compact position/radius bodies only within that borrow. The first two
queries use the unchanged live-array functions and allocate no snapshots. The
third query collects the eligible bodies in original array order, keyed by the
exempt unit (and owner for ordered queries), and reuses them for subsequent
candidate points. Keys changing cause a fresh collection. Position eligibility
remains live/on-map/player<8; ordered eligibility remains live, matching owner
and a current move-family order, including off-map units. Every surviving body
is tested against the same radius sum and exact vector distance. No state,
verdict or index survives the sweep; the borrow prevents mutation of these
fields while the view is in use. The current Sim's valid type-index invariant
still applies. This is not a persistent spatial index or per-tick approximation.

For ordinary nonoverflowing coordinate differences, the maximum absolute axis
is a lower bound on `vector_dist`; a body outside its radius on either axis can
be rejected without division. Large-coordinate cases outside the proved range
and negative radius sums retain the original distance arithmetic. No angle,
candidate order, RNG draw or chosen spot is intentionally changed.

Three new tests compare the axis rejection with exact distance across boundaries
and a dense local grid; compare cached and original scans through mutations of
position, orders, type/radius, owner, health and on-map state; and prove no
snapshot allocation for the first two calls plus reuse thereafter. All12
release tests selected by `radius_` pass. The original scan functions remain
available as the fallback and executable reference. Full diagnostics, contention,
release validation and independent review are pending. Pins and values unchanged.

## Rejected compact-only interpretation and bounded candidate

The first compact-only candidate did not establish a useful ticking improvement.
Two alternating original/candidate walks (`measure.py`, `measurements.json`)
used the same `MallocLargeCache=0` policy: original CPU30.57/31.46s versus
candidate30.69/29.34s; ticking12.97/13.54 versus13.26/12.55s. Wall34.54/34.55
versus32.50/30.46s is confounded by varying decode/I/O times. RSS0.845–0.850
versus0.846–0.855GB is flat. All338 fixture requests and the complete14,748
nonprofiling diagnostic lines match in each walk. The experiment is retained
as `compact-only.patch` and `compact-only-rondata`; those numbers do not prove
a radius CPU gain.

The next candidate filters that temporary list to bodies capable of reaching
an optional sweep bounding rectangle. The extent is a heuristic; **each query
checks membership**, and an outside query calls the untouched original scan.
Overflow constructing bounds disables filtering. Radius changes invalidate
collections alongside owner/exemption changes. The rejection uses i64 axis
bounds and retains bodies whose sum overflows, is negative, or whose coordinate
differences leave the proved nonoverflowing distance range. It therefore does
not assume all large-coordinate distances are nonnegative. Remaining bodies
still use the exact distance predicate and original order. The first two calls
still allocate nothing. No persistent index, test skipping or cached verdict.

The five added tests include exhaustive local distances, conservative rectangle
rejection, empty-list/radius-change invalidation, query fallback, invalid bounds,
and mutations of all eligibility inputs between scopes. Fourteen selected
release tests pass (`bounded-tests-final.log`); final replay build succeeds
(`bounded-build.log`). Independent review and landing validation remain owed.

## Bounded replay measurements

`bounded-measure.py` repeats original/candidate/original/candidate, warm caches,
one test thread, both with `MallocLargeCache=0`. All four exit0 and compare
all338 fixture requests (none missing) plus all14,748 nonprofiling diagnostic
lines exactly. Original wall36.58/30.45s, CPU32.20/29.06s, ticking13.52/12.73s;
bounded wall30.44/28.43s, CPU28.85/28.51s, ticking12.29/12.22s. RSS remains
0.845–0.851GB. The closer second pair saves4.0% ticking and1.9% CPU; the larger
first-pair savings include unrelated setup/decode variability. No memory gain
is established, and this is a coverage-test result rather than a suite10×.

## Contention and complete suite: reject the temporary-list cache

Four/six competing **eight-test samples**, all under the existing shared budget
and `auto` allocator policy, pass all32/48 tests in both modes. Exact fixture
multisets match:1,832/2,748 requests, none missing. Four-suite wall37.96→38.60s,
CPU275.84→272.75s, sampled tree11,590→10,245MiB. Six-suite wall48.79→48.42s,
CPU422.46→415.64s, tree11,220→10,856MiB. CPU gains are only1.1%/1.6%; elapsed
is flat. Aggregate memory peaks also depend on scheduling and are not proof
that the small temporary lists caused a large memory saving.

The complete isolated suite (`full-isolated.py`, both `auto`,8 workers) is the
selection result: all890 tests pass,6 unchanged ignored; exact3,276 fixture
requests match, none missing. Original165.18s elapsed/1,190.57 test CPU seconds;
bounded165.57/1,191.85. Tree8,500→8,229MiB. Whole-suite time and CPU are flat,
and the largest Golden/harness modules are flat/slightly slower. **Reject the
candidate-list cache**: local gains do not justify its added state/keys/bounds.
Preserved sources `bounded-collide.rs`, `bounded-orders.rs` and binary
`bounded-rondata` make the result reproducible; no discarded user files.

A final simpler control keeps the original live scans and applies only the
safe axis rejection. It introduces no temporary list, query cache, or order
changes; its performance and selection are pending below.

## Axis-only control and final selection

`axis-measure.py` runs original/axis/bounded-control/axis with identical fixtures,
allocator policy and full diagnostic comparisons. All four exit0, with the same
338 requests and14,748 diagnostic lines. Original34.54 wall/31.58CPU/13.76s
ticking; axis34.52/31.74/13.82 and34.56/32.20/14.02. Bounded control32.49/29.42/
12.62 confirms its local advantage, but the complete suite above remains flat.
Axis RSS0.851GB is unchanged. Reject the axis-only control too. Its exact-distance
boundary test passed (`axis-tests.log`), but correctness alone does not earn an
optimization. No test was restructured or weakened in any experiment.

The original native sample has14,712 coverage-thread samples, of which463 are
radius-scan leaves (about3.1%). This is a sample of part of one test, not a
whole-suite CPU fraction. It helps explain the limited payoff and does not
justify a broad persistent index with more invalidation obligations.

**Final source and tests are identical to starting tip cb19d62e.** Only the
queue, performance notes, this journal, measurement JSON and the rejected patch
are landing. The patch is documentation for reproduction, not compiled code:
[rejected candidate](../audit/2026-10-16-radius-scans-rejected.patch).
The compact-only patch/binary and axis control source also remain in the
evidence root. Full isolated baseline and bounded manifests each passed890 tests with6
unchanged ignored and3,276 identical fixture requests; no scored word, draw,
value divergence, pin, comparison field, or required fixture moved.10× remains
unmet. No new performance improvement is claimed.

No full release gate is repeated for this documentation/evidence-only landing:
there is no implementation diff. The preceding complete required-fixture gate
on3b7e6846 remains its existing evidence; this turn's isolated runs are explicitly
not represented as a new six-phase gate. Final paperwork/diff/link checks are
recorded below after completion. Earlier optimizations still owe independent
review; these experiments do not self-ratify that debt.

Next measured direction: profile the repeated setup/replay inside Golden and
harness tests. The fresh baseline attributes306.72+253.70=560.42 of1,190.57CPU
seconds to those modules. That selects where to investigate; it does not prove
that setup, ticking or parsing is responsible. Avoid another local optimization
until a representative set of these tests has stage or native-stack evidence.

**For the Loop:** live-unit/radius experiments rejected; words/values unchanged;
890-test full isolated baseline and candidate exit0, full gate not repeated
for the evidence-only landing; commander stays paused, review debt unchanged.

Final paperwork: `zsh tools/guard.sh --built` exits0 (`final-guard.log`):
27 Python checks,28 docs guards,2 no-float guards,11 handoff checks and the
queue ledger pass. `cargo fmt --all --check`, `git diff --check`, local-link/JSON
validation and `git apply --check` for the saved rejected patch also pass.
This is explicitly limited documentation/evidence validation, not a new full gate.
