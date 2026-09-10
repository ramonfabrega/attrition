# PR #3: lab review and adoption map

This is an adoption guide, not a blanket merge recommendation. The lab branch
contains historical mixed commits and experimental runtime work. No lab change
has moved the queue's fidelity scores. Review a group against the owner's
current tip and run its gate; a passing historical run is not a merge proof.

## Start with one reproducible result

From this branch, using the existing install and finalized Logs directory:

```sh
python3 tools/viewer/lab_demo.py "$RON_INSTALL" "$RON_GAMELOG_DIR" /tmp/new-lab-demo
```

Choose a new output directory outside the repository. This builds the release
exporter and runs only Rust replay, with the exact four setup siblings and
run69 trace used by the existing differential test. It launches no original
game, Wine, GUI or capture session. Logs and capture-derived HTML stay local.
Open `focused.html`; `result.json` names the frame, entity, field and both values.
The runner compares all 130 exported frame records between memory and indexed
readers, then checks the focused three records against that window. Whole-input
SHA-256 hashes are checked before and after; the actual exporter binary is also
bound. Install contents are not hashed or bundled.

This is existing city comparator residue, not a newly found production defect
or a deliberately failed acceptance test. If the witness disappears, the
runner fails for investigation rather than choosing a different discrepancy.
It reduces the inspection window, not replay state. A resumable snapshot is
not implemented: late lookup data and correction observations are dependencies,
and sim construction still starts from the original setup.

## Adoption groups and dependencies

| Group | Review anchors | Prerequisites and pilot |
| --- | --- | --- |
| Safe capture reads and incremental replay | `6c956f9`, `c728f80`, `fbe9d05`; `eef695c`, `0caf4f8`, `c4fe241`, `be826c7`; `01bf2bd`, `031d4a5` | Inspect shared Rust hunks separately from lab reports/tools in early commits. Indexed setup depends on the incremental replay state machine and observation accumulator. Compare complete reports and setup records; do not change every consumer blindly. |
| Bounded observation reuse and consumers | `27be422`; `a91ce53`, `79411b5`; `cab564f`, `7423ba1`, `1250501` | Requires indexed capture/setup and its equality tests. The cache has an 8 MiB accounted budget, independent caller copies and metadata identity, not an RSS cap. Road/leader migrations are separate consumer choices. |
| Diagnostic viewer and demonstration | `0e754fd`, `0855e5a`, `a13abf6`; current `tools/viewer/lab_demo.py` | Requires observer replay; the demo additionally exercises the indexed reader. No graphics dependency enters sim. Window export is not snapshot/resume. Retain typed differences and complete-record verification. |
| Verification and scheduling | `6e953eb`, `0ce7e6d`, `b875b34`, `510de2b`, `17adbdb`; current offline demo tests | Explicit install, fixture-request auditing, offline checks and memory monitor belong together before choosing four threads. Direct Cargo defaults to two. The wrapper marker is cooperative, not attestation; the monitor enforces the sampled cap. |
| Determinism and evidence checks | `9c48ef7`, `048573a`; `9106b9a`, `e17ae6e`, `42bb169` | RNG digest and gameplay-activity checks are distinct from fidelity. Trace/receipt validation changes should be checked against finalized retained evidence; do not infer complete native state from accepted receipts. |

These anchors are navigation aids, not a promise of conflict-free cherry-picks.
Use `git show --stat SHA` and inspect the complete diff. In particular, the
viewer/streaming work evolved together, and historical commits sometimes include
more than the shared-file concern named in their title. Documentation relocation
(`534965f`, `ffe2b0e`, `4caea73`) is organizational, not a sim requirement.

## Keep the runtime experiments separate

Live call capsules, native launch probes, unattended capture, and suspended-search
experiments are separate research. They are not needed for the replay demo or
memory gains. Native startup reliability and full headless resumption remain
unresolved. The four local paused drafts are not committed or proposed for
adoption. No live-lane handoff is implied; no message has been sent to Fable.

## Validate the adopted combination

```sh
python3 tools/release_gate.py "$RON_INSTALL" --test-threads 4 \
  --report-dir /tmp/new-adoption-gate
```

Use sequential heavy runs and a fresh report directory. This exercises the full
release suite under a 20 GiB sampled process-tree cap, survey, lint, formatting,
offline regressions and paperwork guards. Fixture auditing proves availability
of observed requests, not completeness of a required corpus. Check the report's
missing fixtures and the expected counts for the adopted revision.

The accompanying [review experiment](2026-09-10-review-demo.md) records the
actual demonstration and a fresh pinned baseline/candidate suite comparison.
That comparison measures the package with its intended scheduling and expanded
coverage; it does not attribute the whole result to any single optimization.

## Follow-on: in-memory continuation

The [owned replay checkpoint experiment](2026-09-10-replay-checkpoint.md) adds
`diff::ReplaySession`. It reuses the existing replay state machine and clones
Built plus the recording stream/cursor together. The [checkpoint export integration](2026-09-10-checkpoint-export.md) now
lets the one-command demo verify three nearby windows from one checkpoint.
Every new process still replays from setup. Adopt this API
only with its recording and corrected-capture continuation tests; it is not a
portable snapshot format or a source-bound replay bundle.

The [controlled figure intervention](2026-09-10-figure-intervention.md) adds a
separate `branch_experiment.py` command and optional checkpoint correction policy.
It preserves the standard replay and explicitly labels the treatment. Its first
result is a verified local null effect, not a reason to remove correction support.
