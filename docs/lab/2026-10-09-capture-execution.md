# Shared execution and opt-in capture compression

Item 1640, Codex (GPT-6), 2026-10-09. User-authorized sequential batch on
`codex/core-loop-investigation`; the commander remains paused. This follows
[the investigation](2026-10-09-core-loop-costs.md).

## Shared widening contract

The run202, run211, run218 and run226 tests use the same Great Lakes initial
state, sibling inputs, capture chain, comparison policy and pool threshold.
Their comparison windows start at block 11400 and end at 15440, 15859, 16711
and 17350. Each previously simulated from zero. The combined test simulates
17350 ticks instead of 65360 and compares 5951 blocks instead of 19764.

`great_lakes_run202_through_run226_are_widened_whole` invokes the same walker
with four checkpoint requests. Each checkpoint copies cumulative first
mismatches, missing fields and counters at that endpoint. Standing rows and
animation changes are restricted to that original test's near-block selection;
animation history still updates on every compared block. Later observations
cannot enter an earlier snapshot. Checkpoint order, bounds and inclusion in
the shared near-block set are checked; an absent checkpoint block fails.

The four original assertion functions retain their historical names. From the
first `pin!` onward their bodies are byte-identical to `e13d1510`. Each holds
its own pin guard; the driver catches failures and runs the remaining groups
before failing with their names. The normal suite executes one test instead
of four. No capture is removed, no window shortened, no floor changed.

`RON_VERIFY_SHARED_WIDENING=1` on the combined test re-executes all four old
walks and checks complete `Widened` equality: first mismatches with their value
strings, missing fields, blocks, leader rows, animation changes, housed rows,
standing rows, and army lists. It also injects one missing field, one missing
block and one extra standing value row per result; each original assertion
group must reject all three. This opt-in migration audit is intentionally
expensive and is not part of every normal gate.

## Initial measurements

Same release configuration (`CARGO_PROFILE_RELEASE_DEBUG=1`), same fixture
corpus and install, one test thread, `tools/memcap.sh 20`:

| Execution | Test seconds | Peak process MiB |
|---|---:|---:|
| Four original tests, one process | 147.59 | 1592 |
| One shared walk, four assertion groups | 55.55 | 1394 |

A 62.4% reduction for this family, not a prediction of the whole-suite change.
Short local builds/guards overlapped these exploratory timings; the work
removed is exact, while elapsed time is a single observation. The complete
required-fixture gate will test the integration. Local logs are under
`target/core-loop-investigation/consolidation-*.log`. The baseline executable
was rebuilt from the unchanged source before measuring; earlier instrumentation
was not used.

## Opt-in compressed-reader contract

`capture::archive::pack(source, destination.rcap)` converts a **finalized**
source. It never changes or deletes the source. The experimental local format
is ours; it makes no claim about the game's own formats and contains no
transcribed game logic. It uses the crate's existing `flate2` dependency.

Layout (little endian): 24-byte header (`RONCAP01`, u32 chunk size 4194304,
u64 logical byte length, CRC32 of the preceding 20 bytes); for each logical
chunk, a u32 compressed length and one independent gzip member; 12-byte footer
(`ENDCAP01`, CRC32 of the header plus all compressed-length fields). The reader
checks metadata, exact file length, bounded compressed lengths and footer at
open. Each decoded member must have the exact expected length, consume exactly
one member, and pass gzip's checksum. A forged expansion stops after one extra
byte. These are corruption checks, not cryptographic authentication.

The source abstraction retains the existing indexed frame/setup/shutdown
parser. Logical offsets and `source_bytes()` still describe original text;
physical length/mtime validate the file and key the index cache. Each archive
reader retains at most two decoded chunks, allocated to 8 MiB plus two bytes,
with a bounded transient compressed buffer. The global cache still holds only
indexes. Ordinary `.txt` inputs remain ordinary files; `.rcap` is explicit.
The full-text `capture::read` API is unchanged and does not read archives.

Publication uses a create-new sibling `.rcap.partial`. After writing the
footer, the writer syncs it, decompresses and compares **every byte** with the
retained source, and rechecks source size/mtime. A hard link publishes the
final path atomically without replacing an existing destination. Only the
writer's own partial link is removed, then the directory is synced. Failed or
interrupted work leaves its partial for inspection. A crash after publication
can leave both links to the same valid file; retries never overwrite either.
This is a finalized-file conversion trial, not a change to live capture output.

For an existing simulation test only, two explicit process-local variables
redirect one indexed source: `RON_INDEXED_ARCHIVE_SOURCE` is its canonicalizable
raw path; `RON_INDEXED_ARCHIVE_PATH` is the new `.rcap` path. Both must be set,
and a missing/invalid archive fails rather than falling back. This override
is a trial mechanism: it does not establish source identity on its own, replace
whole-text reads, or make the fixture-completeness auditor accept an archive
in place of a missing raw capture. Keep the original corpus and verify the
conversion before using it. Neither variable is set during the full gate.

## Compression measurements

Source: existing run721, 257 blocks (3389..3645). Both trial publications are
byte-identical. The original SHA256 still matches the earlier investigation:
`6611a5b01b1faf861ef7be26a3cca6d145ccc51ebd2014bd05d22d55b5e9a1ed`.

- Raw **649903775 bytes**, archive **36937549 bytes**: **17.595× smaller**, or
  **94.32% fewer bytes**. The original is retained, so this trial reclaims no
  disk space. Two local archive publications consume about 70.5 MiB combined.
- Initial pack + sync + full byte verification + publication: **0.848 s**;
  repeated with the two-chunk reader: **0.778 s**, OS peak RSS **18.5 MiB**.
  This excludes the test wrapper's startup/polling interval.
- The initial one-chunk reader's three interleaved runs each: median raw
  **4.80 s**, archive **6.33 s** (+31.9%). Frames cross chunk boundaries and the
  widening reads them twice, evicting the first chunk before its second use.
- Two-chunk follow-up: raw **4.30/4.31/4.37 s**, archive **4.88/4.89/4.89 s**;
  medians **4.31 → 4.89 s** (+13.5%). Raw RSS **819–827 MiB**, archive
  **827–835 MiB**. The additional bounded cache is small relative to the sim
  and other raw inputs. This is not a meaningful sim-memory reduction.

Every run used a fresh process, one test thread, `tools/memcap.sh 20`, and
`diff::coverage_pair::run721_s_word_frame_is_widened_whole`. Order was raw,
archive, archive, raw, raw, archive. All pins passed and all **2513 printed
value/summary rows** matched exactly. Filesystem caches were warm; no machine
cache purge was attempted. Ratios from the earlier Python codec probe differ
because that was a different gzip implementation, not this end-to-end reader.

The two-chunk access-order comparison checked **all 257 frame strings and all
257 frame-plus-sibling strings in each order**, plus exact indexes, setup,
shutdown, logical length and a cached reopen. Its measured read times (seconds,
two slices per frame; equality-check time excluded) were:

| Order | Raw reads | Archive reads |
|---|---:|---:|
| Forward | 0.073 | 0.351 |
| Reverse | 0.073 | 0.446 |
| Seeded shuffle | 0.075 | 0.877 |

Index construction was 0.657 s raw / 0.920 s archive. Combined comparison peak
RSS was 49.4 MiB (OS high-water mark; the one-second sampler missed the peak).
Random reads remain substantially slower than warm raw-file reads, while
bounded in memory and requiring only the requested chunks.

Five new synthetic tests cover empty/exact/cross-chunk inputs, seeks, duplicate
frame labels and late siblings, cached-read mutation detection, bad headers,
lengths, CRCs, footers, truncation, trailing bytes, wrong decoded lengths,
excessive expansion, existing final destinations and interrupted partial files.
All 19 capture tests passed. A final fixed-buffer refinement makes the decoded
allocation bound explicit instead of relying on Vec's growth policy; its tests
also assert the allocation bound. On that final decoder, a fresh six-run series
passed with identical diagnostics: raw **4.26/4.33/4.19 s**, archive
**5.05/5.05/4.81 s**; medians **4.26 → 5.05 s** (+18.5%). Raw RSS
**820–829 MiB**, archive **827–837 MiB**. Across both two-chunk series the
observed penalty is roughly **14–19%**; do not overinterpret the smaller
single-series result. These final logs are `archive-bounded-tests.log` and
`archive-sim-bounded-results.json`.

## Reproduction and evidence

All paths below are relative to this checkout unless absolute. Raw source:
`/Users/rf-studio/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs/gamelog-run721-eastindies-persian-alltech-window-3389-3645.txt`.
Install: `/Users/rf-studio/code/fun/attrition/game`. Retained archives:
`target/core-loop-investigation/run721-trial.rcap` and
`target/core-loop-investigation/run721-publication-trial.rcap`.

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --offline --release -p rondata --example capture_archive
# SOURCE is the raw path above; DEST must be a new, unused local .rcap path.
tools/memcap.sh 20 target/release/examples/capture_archive pack "$SOURCE" "$DEST"
tools/memcap.sh 20 target/release/examples/capture_archive compare "$SOURCE" "$DEST"
# Build the test executable; use the executable path Cargo prints.
CARGO_PROFILE_RELEASE_DEBUG=1 cargo test --offline --release -p rondata --no-run
# With RON_INSTALL set, run the combined Great Lakes test once normally,
# then again with RON_VERIFY_SHARED_WIDENING=1 for the migration audit.
# Run run721's exact test normally and with both archive override variables.
```

Logs under `target/core-loop-investigation/`: `consolidation-baseline.log`,
`consolidation-shared.log`, `consolidation-equivalence.log`, `archive-tests.log`,
`archive-pack.log`, `archive-compare.log`, `archive-sim-results.json`,
`archive-two-{tests,pack,compare}.log`, `archive-sim-two-results.json`,
`archive-integrity.json`, and the `archive-bench*.py` reproduction scripts.
No original assets, capture bytes or derived proprietary exports are in git.

## Recommendation and evidence limits

Keep the shared execution: it preserves the measured evidence and removes
redundant work. Keep compression opt-in: it trades some read CPU/latency for
large storage savings, especially attractive for retained cold evidence.
Measure more capture shapes and migrate all consumers before any corpus
conversion/deletion or automatic producer integration. Hot replay data can
remain raw. Compression does not remove test work and has not made the sim
smaller in memory.

Simulation behavior, draw scores and value floors are unchanged. This is
single-agent executable evidence; independent review remains owed. The frame
reuse patch in the earlier report is still unapplied. Recapturing, replacing
or deleting evidence, and broad test-family migration are separate follow-ups.
Full required-fixture gate remains pending on the final committed tree.
