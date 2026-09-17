# Missing fixture requests are visible in the gate result

## Boundary

A successful test count is not proof that every optional capture was present.
The local release gate now opts the rondata test fixture helper into a request
audit. Each lookup records the requested name, test-thread name, and whether
the file was present. It records metadata only, never capture contents.
A missing lookup may skip a test, omit an optional comparison, or intentionally
probe absence: the report calls it a request, not a measured skipped test.

`python3 tools/release_gate.py INSTALL --report-dir NEW_DIRECTORY` retains
`fixture-coverage.json` and the raw request rows. Without an output argument,
it creates a retained temporary directory and prints its path. Existing output
directories are refused. Only the release-test child receives the audit setting;
survey, clippy and debug paperwork guards do not pollute the result.

The report includes request counts, unique fixtures, per-fixture present/missing
counts and requesting tests. `release_completed` distinguishes a completed
release child from retained partial evidence on failure. `coverage_observed`
is false when no request rows exist; a successful child with no audit is an
error, not a zero-missing success. Malformed audit rows also fail visibly.

By default, missing requests are printed and retained without making every
historical capture mandatory. Add `--require-fixtures` to fail on any observed
missing request. This is an opt-in policy, not a hardcoded required-corpus list.
No file means no bytes checked: presence is not a hash, successful read, or
proof that the fixture is the right version. Unrequested fixtures, tests that
return before lookup, ignored tests, and callers outside testenv::dump remain
outside the observed denominator. `complete_corpus_claim` is always false.

## Implementation

The test-only helper uses RON_FIXTURE_AUDIT_DIR when supplied. Records go to a
per-process TSV file; a process mutex serializes writes from test threads.
Names are UTF-8 bytes encoded in hex, so tabs and newlines cannot create fake
rows. An audit write error panics rather than silently losing coverage evidence.
Ordinary cargo invocations without the variable retain their existing behavior.
The report is additive; no capture is read or altered by auditing its lookup.

Seven Python tests exercise install propagation, failure sequencing, missing
fixture policy, unobserved coverage, malformed rows and partial-run reports.
Two Rust tests cover escaped names, both presence outcomes, and write failures.
A separate real test run pointed only its capture-directory setting at an
empty scratch directory. The test returned successfully after its existing
missing-fixture branch; the audit still recorded exactly one missing run53
request and the test that requested it. The actual corpus was untouched.
Evidence: `/tmp/attrition-fixture-audit-missing/summary.json` and `test.log`.

## Interpretation and next step

Use a completed gate's report to choose which missing requests should be required
for a particular adoption check. Do not derive a universal mandatory list from
one run: conditional branches can hide further dependencies. A future pinned
manifest should distinguish required score fixtures from optional historical
probes and be reviewed against current scoring tests. The new audit supplies
observations for that decision without silently imposing it.

This changes validation reporting, not simulation rules, fixture contents,
parser behavior or capture acquisition. No score moves. The runtime experiment
remains paused and its uncommitted drafts are unrelated.
