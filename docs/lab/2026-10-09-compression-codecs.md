# Compression near raw replay speed

Item 1641, Codex (GPT-6), 2026-10-09, starting at `c595a207`. User-authorized
follow-up to [the gzip trial](2026-10-09-capture-execution.md). One worker;
commander stays paused. No original capture, shared configuration or game
installation is changed.

## Result and decision

On run721's existing whole-record sim test, three interleaved runs per mode:

| Input | Test seconds | Median | Peak process MiB, range |
|---|---|---:|---:|
| Raw | 4.47, 4.19, 4.22 | **4.22** | 820–837 |
| Gzip archive | 4.83, 4.84, 4.75 | **4.83** | 828–837 |
| LZ4 archive | 4.28, 4.25, 4.29 | **4.28** | 828–832 |

LZ4's median difference is **0.06 s / 1.4%**, versus gzip's **0.61 s / 14.5%**
in this series. This supports practically near-raw speed on this workload;
three measurements do not prove statistical equivalence or universal parity.
All **2513 value/summary diagnostic rows** are identical across all nine runs,
and every original test pin passed. LZ4 retains the same bounded two-chunk
cache and does not reduce simulation memory materially.

The safe Rust LZ4 archive is **35106640 bytes** from **649903775 bytes**:
**18.51× smaller**, **94.60% fewer bytes**. Pack, sync, complete byte-by-byte
verification and no-clobber publication took **0.337 s** (OS peak RSS 18.7 MiB).
The earlier Rust gzip archive remains **36937549 bytes**. Compression ratio is
implementation- and dataset-specific: LZ4 being slightly smaller than this
gzip implementation is an observation here, not a general property.

A second capture, run717, gives **7304388 from 117945395 bytes** (16.15×),
verified publication **0.081 s**. Its existing six-missile spline assertion
passes raw and LZ4: raw 0.30/0.31/0.30 s, LZ4 0.32/0.32/0.33 s. The 0.02 s
median gap is 6.7% on this much shorter, reader-heavy test. The claim remains
bounded to the measured workloads.

Keep an explicit LZ4 archive option. Return next to execution consolidation:
it removes repeated simulation, parsing and comparisons together. No recapture
was needed. A corpus conversion/deletion or capture-producer rollout is still
a separate migration; all originals remain available, and default fixture
resolution remains raw.

## Codec-only experiment, not a sim benchmark

Already-installed native libraries, independent 4 MiB chunks, one encode and
median of three decodes per chunk. Timing excludes source I/O and byte-equality
checks; every decode was checked. Codec order rotates per chunk. Native LZ4
uses a reusable output buffer with no format checksum; production Rust LZ4
below adds CRC32. Python gzip/Zstandard return allocated bytes; these API and
copy differences mean the microbenchmark ranks candidates, not their exact
Rust end-to-end cost.

| Codec, run721 | Compressed MB | Ratio | Encode s | Decode s |
|---|---:|---:|---:|---:|
| gzip-1, zlib decoder | 31.24 | 20.80× | 0.772 | 0.094 |
| Same gzip, libdeflate decoder | 31.24 | 20.80× | 0.758 | 0.071 |
| Native LZ4 | 35.11 | 18.51× | 0.141 | 0.035 |
| Zstandard fast -3 | 24.00 | 27.08× | 0.157 | 0.095 |
| Zstandard 1 | 16.86 | 38.55× | 0.180 | 0.105 |
| Zstandard 3 | 16.21 | 40.09× | 0.202 | 0.104 |
| Zstandard 6 | 14.20 | 45.76× | 0.649 | 0.098 |

Run717 showed the same broad ordering: LZ4 16.15×, 0.007 s decode;
Zstandard 3 35.41×, 0.021 s decode; gzip/zlib 18.35×, 0.019 s decode.
These are text captures, not binary snapshot benchmarks. Source SHA256s match
the earlier investigation for both files; sizes and mtimes stayed unchanged.
No alternative codec packages were globally installed for this probe.

Candidates came from primary documentation: [LZ4](https://lz4.org/),
[Zstandard](https://facebook.github.io/zstd/),
[libdeflate](https://github.com/ebiggers/libdeflate), and
[lz4_flex's safe Rust configuration](https://github.com/PSeitz/lz4_flex).
Their published benchmark numbers were not substituted for measurements here.

## Implementation and format contract

`capture::archive::pack` still writes the existing gzip format. New
`pack_with_codec(..., Codec::Lz4)` and the example driver's **`pack-lz4`**
select LZ4 explicitly. `IndexedCapture::open` accepts either archive version;
its existing explicit two-variable test override works unchanged. No new
simulation dependency, float arithmetic, unsafe allowance or original logic
is introduced. The sole new dependency is **lz4_flex 0.12.2**, locked, with
`default-features = false` and explicit `std`, `safe-encode`, `safe-decode`.
The resolved feature tree confirms those features and no optional frame codec.

The v1 gzip layout and verification are retained. V2 LZ4 uses header magic
`RONCAP02`, footer magic `ENDCAP02`, and the same fixed 4 MiB logical chunk
size and 24-byte header. Each chunk is:

1. little-endian u32 compressed size;
2. little-endian u32 CRC32 of the exact original chunk bytes;
3. an independent LZ4 block (no prepended-size or LZ4 frame wrapper).

The existing footer CRC now covers the header and both u32 fields for each
chunk. The reader bounds chunk count against physical file size, validates
metadata CRC and exact footer placement, checks compressed sizes, and decodes
into an output slice bounded by the expected logical size. It requires exact
decoded length and matching decoded CRC before caching any bytes. The cache
remains at most two buffers, 8 MiB plus two bytes in capacity. CRCs detect
accidental corruption; they are not cryptographic authentication.

The create-new partial, full decoded-byte verification, source-metadata checks,
atomic no-clobber hard-link publication and directory sync are unchanged.
Old archives are not rewritten. Archive format claims here describe our own
container and are exercised by synthetic tests, not inferred game formats.

All five prior archive tests now exercise **both** formats, with identical
assertions: empty/exact/cross-chunk lengths, forward/backward seeks, allocation
bounds, duplicate labels and siblings, mutation invalidation, corrupt/truncated
metadata and payloads, trailing bytes, expansion bounds, existing destinations
and interrupted partials. One new test forges a valid metadata envelope with
a wrong decoded CRC and requires rejection without caching the failed decode.
All **20 capture tests passed**; all-target clippy passed.

## Full-capture access checks

All 257 run721 frames and all 38 run717 frames, including frame-plus-sibling
slices, were byte-identical to raw in forward, reverse and seeded shuffled
order. Setup, shutdown, frame indexes, logical byte lengths and a cached reopen
also matched. These passes overlapped a build, so their elapsed times are
exploratory, not isolated performance benchmarks.

On run721, index scan was 0.686 s raw / 0.766 s LZ4. Forward reads were
0.078 / 0.119 s, reverse 0.078 / 0.140 s, shuffled 0.080 / 0.237 s (two slices
per frame, comparison time excluded). Random reads still cost more than warm
raw reads even though the overall simulation test is nearly unchanged.

## What else is worth considering

- **Zstandard for colder evidence:** it was substantially smaller than LZ4
  on both text files, with a fast native decoder. Not implemented in the
  replay reader here; native microbenchmarks are not sufficient to adopt it.
- **Faster gzip backend:** libdeflate measured faster on the exact same
  gzip bytes. No format change would be needed, but it would require another
  dependency/backend evaluation; LZ4 already met this trial's practical goal.
- **Persist the validated frame index at archive creation:** a fresh reader
  currently decompresses and scans the entire capture to discover offsets.
  This can eliminate startup work, but isn't new: item 1571 measured sidecars
  at 1.42× and rejected them below its 1.5× target; the in-process cache already
  shares indexes. An index inside an immutable compressed archive changes the
  packaging/invalidation tradeoff, not that earlier measurement. It remains
  unimplemented, and benefits must be measured for whole-suite versus cold opens.
- **Frame-aligned chunks:** could reduce unnecessary neighboring-byte decode
  and avoid straddling reads. Needs a bounded policy for oversized frames and
  preservation of setup/sibling offsets. Not tested here.
- **Read/parse each frame once:** the earlier frame-reuse experiment already
  showed a real gain for Great Lakes. This complements compression and shared
  execution; it is still separate from the adopted change here.
- **Lossless frame deltas or reversible column/token encoding:** plausible
  because nearby text frames repeat extensively. Random access would need
  checkpoints and exact reconstruction of every original byte. Higher
  implementation and validation cost than the measured codec switch; defer.
- **A disposable parsed cache:** could avoid the parser entirely for hot data,
  but must be keyed by source content and parser/schema version and checked
  against raw parsing. It must not replace raw evidence or omit fields used
  by coverage checks. This is a proposal, not measured here.

## Reproduction and evidence

Local artifacts in `target/compression-followup/`: `codec_probe.py`,
`codec-results.json`, `codec-probe.log`, `run721-lz4.rcap`, `run717-lz4.rcap`,
`pack721.log`, `pack717.log`, `compare721.log`, `compare717.log`, `e2e.py`,
`e2e-results.json`, per-run `e2e-*.log`, `secondary.py`,
`secondary-results.json`, `final-reader-tests.log`, `clippy.log`.
The raw sources remain in their existing `/Users/rf-studio/ron-data/` paths.

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --offline --release -p rondata --example capture_archive
# SOURCE is an existing finalized capture; DEST is a new, unused .rcap path.
tools/memcap.sh 20 target/release/examples/capture_archive pack-lz4 "$SOURCE" "$DEST"
tools/memcap.sh 20 target/release/examples/capture_archive compare "$SOURCE" "$DEST"
```

The exact sim tests are `diff::coverage_pair::run721_s_word_frame_is_widened_whole`
and `diff::coverage_pair::run717_s_missiles_fly_their_spline_s_count`, with
`RON_INSTALL=/Users/rf-studio/code/fun/attrition/game`. Repeat raw and with
`RON_INDEXED_ARCHIVE_SOURCE="$SOURCE" RON_INDEXED_ARCHIVE_PATH="$DEST"`.
End-to-end measurements used fresh processes, one test thread, no concurrent
builds, warm filesystem caches and `tools/memcap.sh 20`.

Single-agent executable evidence; independent review remains owed. Draw scores
and value floors are unchanged. Full required-fixture release gate on committed
`906d77de` **passed, exit 0**, all six steps: offline (277 tests), clippy,
formatting, survey, release and guard. Release: fixed 13, rondata 782 passed
(five existing ignored, 592.85 s), sim 1476 passed (2.23 s), three doc tests.
The fixture audit observed 2963 requests across 386 unique fixtures, zero
missing; it does not claim every corpus file was exercised. Peak tree RSS
12935 MiB, largest process 12902 MiB. The full gate used raw-default fixtures;
compressed-reader evidence is the separate trials above. No corpus migration
is claimed. Gate logs and JSON reports are retained under
`target/compression-followup/{release-gate.log,gate-report/}`. See the
[item journal](../journal/2026-10-09-item-1641.md) for the landing handoff.
