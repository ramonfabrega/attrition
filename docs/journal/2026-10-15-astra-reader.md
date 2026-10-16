## 2026-10-15 — Astra reader bottleneck exploration

Branch: `codex/astra-windowed-reader`, based on `0efe53b7`. This work follows
the first step in `/Users/rf-studio/.claude/jobs/d388900a/tmp/astra-reader-brief.md`.
Pins and simulation behavior are unchanged. No subagents were used.

### Baseline and findings

The brief's baseline timing report (`docs/audit/2026-10-15-suite-timings.txt`)
measured 2,762 seconds across 858 tests. The two targeted tests took 137.36 s
(`every_key_the_dump_prints_is_read_or_pinned`) and 202.00 s
(`every_parsed_field_is_compared_by_the_instrument_or_pinned`). A direct
`/usr/bin/time -l` baseline for the latter peaked at 2,694,545,408 bytes RSS.
The frame-offset index and windowed reader were already present in memory;
there was no persistent index between test processes.

### Changes

- `crates/rondata/src/capture/indexed.rs`: persist only frame/setup offsets
  beside captures in metadata-keyed, versioned sidecars. Validate source
  metadata, bounds, ordering, and checksum when loading; create via temporary
  file and rename. New source metadata gets a new sidecar without deleting an
  older one. Gaps between frame ranges are valid because sibling dump records
  can sit between frames. Tests cover a corrupt offset/checksum, sidecar
  persistence, changed-source invalidation, and sibling pairing.
- `crates/rondata/src/diff/coverage.rs`: execute the 63 coverage windows one
  at a time and drop each detailed `Widened` result after accumulating the
  existing recorder/block evidence. The full compared-pin walk and its order
  remain asserted.
- `docs/DATALAYER.md`: document the sidecar format and invalidation behavior.
- `docs/audit/2026-10-15-astra-reader-suite-timings.txt` and
  `docs/audit/2026-10-15-astra-reader-suite-memory.txt`: preserve the full
  per-test timing and RSS sweep.

### Measurements

The warm, separate-process targeted key test fell from 136.05 s to 32.60 s
(about 76% faster); the warm run loaded 190 persisted indexes, built none, and
peaked at 113,164,288 bytes RSS. Its cold sidecar-building run took 133.06 s.
The full sequential suite sweep measured 2,111 s over 866 tests, versus 2,762 s
over 858 in the baseline report: 651 s less (23.6%). The two target rows were
34.42 s and 172.38 s. The sweep's RSS report gives 108 MiB and 2,532 MiB for
those tests. A separate post-change direct run of the high-memory test peaked
at 2,493,906,944 bytes; these observations suggest a modest reduction, but
RSS varied substantially between runs and does not establish a reliable
memory improvement. Suite-level memory remains a concern: individual unrelated
tests in the sweep peaked above 4 GiB.

The sweep's original compiled binary recorded one isolated failure in
`diff::floors::tests::the_compared_pin_walks_every_open_ai_word`: it searches
the test source for literal `name()` calls, while the first sequential rewrite
used function pointers. Window entries were changed to lazy closures retaining
the literal calls. Rebuilt follow-up: that exact guard passed (1/1). The
earlier focused golden test also exposed an overly strict validator assumption
that frame ranges must be contiguous; sibling `FULL DUMP` spans disprove that.
The validator now permits gaps but still rejects overlap/order errors, and the
chapter-four golden replay passed after the correction. The full sweep then
completed; its other game/capture cases passed.

### Validation and landing

Focused `capture::indexed::tests`: 13 passed. The corrected source guard
passed (1/1); `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`,
`tools/guard.sh --built`, and `git diff --check` passed. The guard reports an
existing note that queue item 7 is ledgered and live again.

Implementation checkpoint: `2c433994`. The first required gate attempt exited
1 at `offline_tests.py`, before clippy, formatting, survey, release, or guard.
Its only actual failure was
`test_the_pdb_default_resolves_the_main_checkout_from_a_worktree`: the gate
sets `RON_INSTALL`, while the test intended to exercise the fallback before
clearing it. The isolated test setup now sets an empty `RON_INSTALL` for that
fallback assertion (`723c899d`). The exact test passed 1/1; the gate's offline
suite passed on retry.

Full gate on `723c899d` exited 0: all 6 steps ran (offline, clippy, fmt,
survey, release, guard); 13 fixed tests, 866 rondata release tests, 2,584 sim
tests, and 3 rondata doc-tests passed. Required-fixture audit: 3,186 requests,
0 missing. The four-thread memory cap observed 11,503 MiB peak across the
tree, 11,470 MiB in its largest process, under the 20 GiB cap. The committed
gate verdict is recorded separately after this journal update.

### CPU profile follow-up — 2026-10-15

The brief's next proposal, a persistent parsed-record store, did not match the
active bottleneck. A 20-second `/usr/bin/sample` of the release
`every_parsed_field_is_compared_by_the_instrument_or_pinned` test recorded
1,808 thread samples. `gamelog::fill` appeared in 13 top-of-stack samples;
repeated region-grid work was much hotter: the `land_size` cell-count fold
appeared in 165, and `World::region_size` in 133. The process footprint during
the sample was 1.2 GiB, peaking at 1.3 GiB; this is a sampled point, not a
whole-run memory measurement.

The parsed store is therefore deferred: this profile gives no evidence that
text parsing is a material share of this test's CPU. The exploration instead
caches exact per-region cell counts in `sim::world::World`, updated only when
`set_region` changes membership, and makes territory victory use that count.
This crosses the brief's sim boundary deliberately to address the measured
CPU hotspot. Region IDs, terrain, counts, and all simulation comparisons stay
the same. A differential unit test compares the maintained counts against a
full scan after assignments, reassignment, repeated assignment, and an
unassigned cell is assigned.

The focused coverage test passed after the change in 167.63 s. A quiet direct
baseline took 175.07 s, an observed 7.44 s (4.2%) reduction. The previous
suite sweep recorded 172.38 s, and a profiled baseline took 184.40 s; this
spread means the focused result suggests a modest win but does not establish
a stable suite-wide improvement. The implementation checkpoint is
`f36f1393`. Its first full-gate attempt exited in offline validation because
sandbox restrictions prevented `ps` execution and macOS LaunchServices could
not open fixture apps; the gate did not reach the release suite. No source or
fixture test was weakened. The required gate was rerun with host permissions
on the unchanged committed tree and passed: all 6 steps, 13 fixed tests, 866
rondata tests, 2,585 sim tests, and 3 rondata doc-tests; fixture audit 3,186
requests, 0 missing. The four-thread memory cap observed 12,543 MiB across
the tree and 12,510 MiB in the largest process, below 20 GiB. Gate log:
`/tmp/attrition-region-cache-release-gate-elevated.log`. The final gate
verdict is recorded in a follow-up commit. The profile is retained at
`/tmp/attrition-coverage-profile-elevated.sample.txt`.

### Shutdown-test memory follow-up — 2026-10-15

The previous RSS sweep's two largest tests were the East Indies and Great
Lakes shutdown-oracle comparisons. On the current baseline tree, each called
`Log::parse` on each whole capture to obtain both setup observations and the
trailing shutdown dump, peaking at 4,888,231,936 B and 4,238,639,104 B RSS.
Those tests only need setup/replay observations and the final frame's tail.

Both now open `IndexedCapture`, read the shutdown tail separately, and build
their simulation setup through `with_replay_initial`. Great Lakes obtains its
map siblings through `with_sibling_initials`; East Indies streams run38's
replay observations as well as its setup. The first attempt that read only
run38's setup omitted its frame seeds and clocks and failed on run55's closing
state. The helper was corrected to include the indexed observations; all nine
East Indies and all five Great Lakes rows then retained their exact compared,
unlinked, torn, and off counts.

One-test process measurements (`/usr/bin/time -l`, same release test binary and
install) are recorded in
`docs/audit/2026-10-15-astra-shutdown-indexed.txt`. East Indies fell from 15.41
to 9.73 s, 4,888,231,936 to 993,017,856 B RSS (79.7% lower), and 372.2B to
225.0B retired instructions (39.5% lower). Great Lakes fell from 8.03 to 5.24
s, 4,238,639,104 to 1,017,233,408 B RSS (76.0% lower), and 181.5B to 117.5B
instructions (35.3% lower). These are single-run measurements, but the memory
reduction is large and the original value assertions remain intact. Focused
release tests passed for both tests.

All six focused rows passed. The implementation checkpoint is `e6d98bd1`.
The required gate on that committed tree passed all 6 steps: 13 fixed tests,
866 rondata tests, 2,585 sim tests, and 3 rondata doc-tests; fixture audit
3,187 requests, 0 missing. The rondata release run finished in 496.11 s.
Peak memory was 11,056 MiB across the tree and 11,022 MiB in its largest
process, below the 20 GiB cap. The previous full gate on `f36f1393` peaked at
12,543 MiB across the tree, so this run observed 1,487 MiB less at suite
level; the per-test RSS reductions above are the more direct evidence. Gate
log: `/tmp/attrition-shutdown-indexed-release-gate.log`. The follow-up commit
records this verdict.

### For the Loop

The windowed index now reduces the two biggest observed one-test RSS peaks by
76–80% while also reducing their CPU time by about a third. The next memory
target was two unit tests that only inspect their captures' opening units.

### Unit-test memory follow-up — 2026-10-15

The tests `every_dumped_unit_has_crew_size_plus_one_figures` and
`the_walk_gives_every_dumped_guy_its_own_piece` each read ten full captures,
then use only the opening unit list. They now parse `IndexedCapture`'s replay
setup slice and call `replay_initial`, without indexing or parsing frame
observations that these assertions do not inspect. Errors opening or reading a
fixture still fail; absent fixtures and absent initial states retain their
existing handling. All named captures and per-unit assertions remain.

Focused release tests passed with 120 units checked (100 single-figure, 20
two-figure) and 126 figures checked across 12 distinct graphic pieces. The
before/after timings, RSS, instruction counts, and reproduction logs are in
`docs/audit/2026-10-15-astra-unit-setup.txt`. Crew-size checks fell from 8.61
to 0.95 s, 4,476,321,792 to 327,352,320 B RSS (92.7% lower), and 194.6B to
22.8B retired instructions (88.3% lower). Graphic-piece checks fell from 7.74
to 0.96 s, 4,407,672,832 to 368,181,248 B RSS (91.6% lower), and 190.9B to
22.8B instructions (88.0% lower). Full required-fixture gate validation is
pending on the committed implementation checkpoint.

### For the Loop

Four high-RSS tests now use indexed reads. The full gate following the first
pair recorded 11,056 MiB peak across the tree, 1,487 MiB below the previous
gate's 12,543 MiB observation. After this landing, profile
`diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned`
again: it remains the longest single rondata test (167.63 s in the recent
focused run) and used 2.5 GiB in the earlier per-test sweep. The region-count
cache was a modest initial win; measure the new hotspot before choosing the
next change. The parsed per-lobby store remains deferred until the profile
shows text parsing is a material CPU cost.
