# Bounded sibling text, explicit installs, and finalized trace checks

## Sibling retention: measured tradeoff

Only run71's 5,000-frame replay test migrates in this tranche. Its primary
capture remains unchanged. A scoped helper opens the four sibling captures
through IndexedCapture, retaining setup text and owned observations rather
than all four complete source Strings. The helper nests each initial state's
lifetime around the callback. It has no raw-text cache and no unsafe code.
Observation arrays still grow with observed data; this is bounded source-text
retention, not a claim of constant total memory.

A permanent corpus check compares every Initial field from each old reader
with its indexed counterpart after clearing only `frame_bodies`, the audit
series. A separate temporary assertion ran both complete run71 reports and
compared them for exact equality, then restored the test source. Both passed.
`borrow_from_siblings` uses the retained observations, not that audit series;
replacing the siblings with prefix-only text would lose frame seeds and clocks.

Uninstrumented release executable, exact same run71 filter, one sample each:

| Metric | Whole siblings | Indexed sibling text |
| --- | ---: | ---: |
| Maximum RSS, bytes | 2,225,340,416 | 1,494,859,776 |
| Elapsed seconds | 7.47 | 8.58 |

This saves 730,480,640 bytes of isolated peak RSS but takes 1.11 seconds longer.
It is a memory/time tradeoff, not a speedup or a whole-suite improvement. The
migration stays limited to this high-retention replay test. Evidence:
`/tmp/sibling-run71-before.log`, `/tmp/sibling-run71-after.log`,
`/tmp/sibling-equivalence.log`, `/tmp/sibling-complete-report.log`.
No fidelity floor changed. Other callers of sibling_texts remain candidates,
not automatically eligible migrations.

## Explicit-install release gate

`python3 tools/release_gate.py INSTALL` refuses an absent Data/rules.xml before
launching a child, then runs the data survey, the release suite under the
20-GiB process-tree cap, clippy with warnings denied, formatting, and paperwork
guards. Every child receives the canonical explicit RON_INSTALL path. A survey
failure stops the chain before tests. This addresses the Codex worktree lookup
gap documented in the preceding fixture-memory report without adding a guessed
machine-specific lookup to the crate.

Three authored orchestration tests verify preflight refusal, install propagation
to every child, and stopping on a failed survey. The runner still honors the
existing RON_GAMELOG_DIR/default capture location. It does not certify that
every optional historical capture exists; missing-corpus coverage remains a
separate contract. Existing direct cargo invocations are not intercepted.

## Finalized trace boundary

Trace::parse remains a permissive diagnostic reader. Trace::parse_finalized
rejects incomplete headers/records, versions other than 1 and 2, and positive
reported loss counts. Trace::read now uses that boundary for recognized RONT
files; malformed evidence yields InvalidData, while unrecognized files retain
Ok(None). This is framing/loss validation, not a proof of complete capture.
A header-only trace can be structurally valid; neither an abrupt end on a full
record nor missing expected frames is certified by these checks.

The evidence is the repository-owned writer protocol: tracer.c emits version
2, and flush_locked emits INFO kind 5 / I_DROPPED 14 with its count in slot 2.
Existing authored version-1 fixtures establish supported legacy decoding. No
original executable or proprietary record content is embedded in these tests.
Authored cases cover both versions, all 31 partial-record tail lengths, unknown
versions, short headers, positive/zero loss notices, and file-read propagation.

Existing CLI and test callers that discarded read errors no longer turn invalid
supplied evidence into a missing optional trace. Missing paths can still be
optional where they already were; a present invalid trace fails visibly.
Before changing the boundary, a read-only scan of 87 stored rontrace log files
found no version, alignment, or reported-loss violation. Metadata result:
`/tmp/finalized-trace-preflight.json`. This is not a complete-capture verdict.

## Remaining work

Measure other sibling consumers before trading speed for memory. Consider
owned query products only with an explicit size/lifetime budget. Separately,
a required-corpus manifest could make missing optional fixtures visible in
the release result; the new install runner does not solve that. Frame coverage,
semantic trace versions, and matching capture/trace identity remain distinct
from the new finalized framing checks. The runtime experiment stays paused.
