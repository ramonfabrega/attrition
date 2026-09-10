# Bound capture retention in leader-window tests

## Consumer and change

The Great Lakes leader helper used four complete sibling capture strings to
construct the simulation, then kept those strings and their parsed logs in
scope while ticking to the requested window. The two active consumers check
run84 and run91's complete mapped leader records; an ignored diagnostic probe
uses the same helper.

Construction now uses the existing `with_sibling_initials` callback, retaining
bounded setup text and owned observations instead of whole sibling text. The
primary run53 capture is still read in full. Construction returns an owned
`Built`, releasing all capture inputs before ticking. No persistent cache,
parser change, new dependency, or change to the field assertions is introduced.
Other sibling consumers are unchanged.

## Exact output check

Temporary instrumentation wrote the complete ordered `Kept` maps before and
after the change. Both windows matched byte-for-byte: 54,240 field values for
6950–7029 and 58,308 for 7514–7599, totaling 112,548. Snapshots are outside git:
`/tmp/leader-loading-{before,after}-snapshots/`. SHA-256 values of the identical
serialized maps are:

- 6950–7029: `0a52a8be5f2eb370e108e0ef4b5b6dea8f35c113f7d7eadf4bf739495becd643`
- 7514–7599: `8892ba67ecbf9a520125a8501cb191b97d32c0a57f4db1e479e6a731bad6076a`

Instrumentation was removed. Existing tests still compare the same whole
mapped records and pin the same named residue against the original. The
existing sibling regression checks every retained Initial field against the
whole-text parser, excluding its explicitly omitted audit frame bodies. These
checks establish this consumer's output, not exhaustive simulation state.

## Isolated measurements

Clean release binaries were built from the previous committed helper and the
new helper. macOS `/usr/bin/time -l` ran the same two active leader tests with
one test thread and explicit RON_INSTALL, sequentially in before/after/after/
before order, with no snapshot instrumentation. Compilation is excluded.

| Sample | Version | Peak RSS, bytes | Test seconds | Process wall seconds |
| --- | --- | ---: | ---: | ---: |
| 1 | Before | 1,570,766,848 | 5.73 | 6.14 |
| 2 | After | 855,097,344 | 6.52 | 6.80 |
| 3 | After | 1,066,663,936 | 6.48 | 6.48 |
| 4 | Before | 1,562,132,480 | 5.68 | 5.69 |

This is a memory/time tradeoff: lower isolated peak RSS, about 0.8 seconds more
test time across the pair. RSS varies between runs. These four observations
are not a statistical suite benchmark, and do not imply a corresponding
full-suite peak reduction. Logs: `/tmp/leader-clean-{1-before,2-after,3-after,4-before}.log`.
Earlier exploratory measurements with disabled snapshot instrumentation are
kept separately; an initial sandboxed timing attempt passed the tests but
could not query RSS. Final measurements used expanded statistics permissions.

This scoped tradeoff is retained for the memory-heavy test loop. Broader
migration requires separate measurements; repeated scanning/decoding remains
a speed target. No fidelity score or runtime behavior changes. The paused
runtime drafts remain untouched.

## Full gate

The explicit-install gate passed 27 offline Python tests, 275 rondata tests
(one ignored), 824 sim, 13 fixed and three doctests, plus survey, clippy,
formatting and guards. The complete sibling-field equality regression passed.
Fixture coverage remained 550 requests with none missing. Rondata test time
was 241.09 seconds; sampled process-tree peak was 8991 MiB under 20 GiB.
Those suite figures are not an isolated causal measurement of this change.
Log: `/tmp/leader-memory-final-gate.log`; fixture report:
`/tmp/attrition-gate-leader-memory-20260909/fixture-coverage.json`.
