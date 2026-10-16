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

### For the Loop

Persisting offsets removes repeated whole-capture scans between test processes;
the warm key-reader test fell about 76%, but the parser still reads and parses
the requested windows each time. A parsed per-lobby store remains the next
untested edge from the steering brief. Do not infer a robust memory win from a
single high-RSS comparison.
