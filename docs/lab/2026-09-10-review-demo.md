# A reproducible lab introduction

## Question and experiment

Can an existing full-input replay discrepancy become a small, inspectable
headless regression? The implemented demonstration runs the same run69 capture,
trace and ordered four siblings as the retained test, using the existing
exporter. It compares a 130-record window starting at frame 1900 between memory
and indexed readers, selects the observed city `0/2000` `peasant_dist` residue,
then reproduces it in a three-record indexed window. Complete frame records,
not just the selected difference, must agree. Broad-reader notes, world,
correction counts and applied-order counts must agree too.

The source comments described waypoint discrepancies, but the inspected current
window had none. It did contain city residue, including original 2 versus Rust
0 for `peasant_dist`. The demo follows current evidence rather than treating a
historical comment as a guaranteed failing case. This is existing comparator
residue, not a new acceptance failure or a fidelity-score advance.

## Reproducibility boundary

`tools/viewer/lab_demo.py` accepts install, Logs and a new output directory
outside the repository. It builds the release exporter, hashes all six capture/
trace inputs before and after, and verifies the actual exporter binary is
unchanged. It writes the commands, measurements and hashes only after successful
verification. Existing output directories are refused. Subprocess failures
retain logs and cannot produce a success report.

The install is external, not bundled or content-bound. HTML and JSON artifacts
contain local capture records and must not be committed. This is diagnostic
window reduction, not autonomous replay-state minimization: initialization,
whole-source correction observations and replay from setup are still required.
The report lists those dependencies explicitly. No native process or capture
lane is used. The paused runtime drafts remain untouched.

Two authored tests check complete-record mismatch refusal and missing/agreed
witness refusal; they are included in the reviewed offline gate. Existing export
verification recomputes coordinate and issue counts. A deliberately changed
record and agreeing witness are rejected rather than accepted as a reproduction.

## Combined benchmark method

The pre-lab baseline is the immutable merge base `bbd51735aba5384b34b6f9a22b884fe6215e9de9`,
exported with `git archive` into `/tmp/attrition-review-baseline-bbd5173`.
The candidate's Rust code is `da85116`; this turn adds only the demo, its offline
tests and adoption documentation. Both release suites use the same explicit
install and default local Logs directory, with no concurrent heavy workload.
Compilation is completed before the release-test timing. Baseline uses its
supported two-thread policy; candidate uses the monitored four-thread option.
Both run under the current 20 GiB process-tree monitor.

This is a package/policy comparison with expanded candidate coverage, not a
same-test-count microbenchmark or proof that any single optimization caused the
speedup. Baseline has no observed-fixture audit; no identical request set is
claimed. Baseline is the full release suite, not a retrospective run of the
new Python gate. Candidate validation additionally runs survey, offline tests,
lint, formatting and guards. Single runs do not establish statistical bounds.

## Demonstrated result

The local run succeeded at `/tmp/attrition-lab-demo-final-20260910`:

- Witness: frame 1900, source index 1899, city `0/2000`, `peasant_dist`,
  original `2`, Rust `0`.
- Both 130-record exports agree in complete frame records and checked provenance.
- The three focused records agree completely with the broad export; the witness
  is identical. All six input hashes and the exporter hash remained unchanged.
- Broad indexed: 37,690,822 bytes, 5.816 seconds. Broad memory: 37,690,820 bytes,
  4.804 seconds. Focused indexed: 912,048 bytes, 4.836 seconds.

Artifact size falls about 97.6%; this is not an equivalent replay-time reduction.
These are single ordered observations, including verification in the timed
export helper, not a controlled reader-speed benchmark. The three-record
artifact is an inspection product and remains dependent on full replay inputs.
The README identifies the field-filter action; city rows are not unit markers.

Re-run the documented one-command demo with a fresh output directory to verify
the actual hashes and exact values locally. No capture content is committed.

## Fresh package comparison and validation

| Revision / policy | Rondata tests passed | Rondata seconds | Sampled tree peak MiB |
| --- | ---: | ---: | ---: |
| Pre-lab `bbd5173`, two threads | 248 | 287.57 | 10,869 |
| Candidate `da85116` Rust plus this demo, four monitored threads | 280 | 122.07 | 13,307 |

Both had one ignored test and zero filtered tests. The candidate's rondata
elapsed time is about 2.36 times faster in this package/policy sample, with
32 additional rondata tests. This is a reduction of about 57.6% in this phase's
elapsed time, not total development time or a same-width implementation effect.
Peak memory is higher for the wider candidate; both remain under 20 GiB.

Baseline also passed 821 sim tests, 13 fixed tests and three doctests; the
candidate passed 824 sim, 13 fixed, three doctests, 36 offline Python tests,
install survey, clippy, fmt and paperwork guards. All 557 observed candidate
fixture requests were present. The baseline lacks that audit. Baseline Cargo
release-test process wall time was 290.83 seconds; do not compare that directly
with the candidate's entire multi-stage gate.

Evidence: `/tmp/lab-review-baseline-build.log`,
`/tmp/lab-review-baseline-suite.log`, `/tmp/lab-review-demo.log`,
`/tmp/lab-review-candidate-gate.log`, and
`/tmp/attrition-gate-review-demo-20260910/fixture-coverage.json`.

Recommended next research: define an owned snapshot and its correction/order
cursors, then prove continuation from that snapshot equals uninterrupted replay
before attempting case minimization. A short HTML window alone cannot supply
those missing semantics. Fable can adopt the established reader/gate/diagnostic
groups independently of that future work.
