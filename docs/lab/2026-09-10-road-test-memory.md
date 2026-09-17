# Release road-test capture inputs before simulation

## Opportunity and change

The road-ring and market-road regressions retained four whole sibling captures
while ticking their constructed simulations. The market test additionally
loaded its comparison capture while those inputs remained resident. The four
siblings contain about 1.09 GB of text (L44); their bounded replay setup and
owned observations already have complete-field equality coverage.

Both tests now construct their simulation inside `with_sibling_initials`.
The primary capture and its parsed initial are scoped inside that callback,
so all construction inputs are released before ticking. The primary input is
still read in full during construction. The market test retains its existing
pasture-trace correction. Simulation logic, tick counts, comparison parsing,
and every assertion remain unchanged. No capture or gameplay score changes.

## Isolated measurements

Clean release test binaries before and after the edit ran sequentially, with
one test thread and explicit RON_INSTALL. Each test used before/after/after/
before order in fresh processes; compilation was outside measurement. Both
versions include L44's observation cache, initially cold in each process.
All eight runs passed with no fixture-skip messages.

| Test | Sample | Peak RSS bytes | Test seconds |
| --- | --- | ---: | ---: |
| Road rings | 1 before | 1,821,474,816 | 3.20 |
| Road rings | 2 after | 1,019,232,256 | 3.58 |
| Road rings | 3 after | 1,016,365,056 | 3.62 |
| Road rings | 4 before | 1,811,628,032 | 3.16 |
| Market road | 1 before | 3,213,246,464 | 5.82 |
| Market road | 2 after | 1,927,806,976 | 6.23 |
| Market road | 3 after | 1,898,184,704 | 6.21 |
| Market road | 4 before | 3,237,543,936 | 5.79 |

The market-road peak falls about 40%, with about 7% more cold execution time.
The road-ring samples likewise trade lower peak memory for more cold setup
work. Bounded scanning does additional I/O; cache reuse across tests may help,
but these measurements do not establish a full-suite speedup or memory bound.
Peak RSS is measured by `/usr/bin/time -l`, not estimated from file sizes.
Logs: `/tmp/road-memory-run*-{1-before,2-after,3-after,4-before}.log`.
The baseline source is branch tip `0c6446e`.

## Remaining opportunities

Thirty-one `let texts = sibling_texts()` call sites remain across seven diff
modules after this change. This is a source inventory, not a ranking by cost.
Consumers that inspect sibling logs after construction need individual lifetime
analysis; mechanically replacing all of them would be unsound. Whole primary
captures remain another opportunity where only setup is consumed. Existing
whole-text versus indexed equality regressions should remain independent.

## Full gate

The explicit-install monitored four-thread gate passed: 34 Python tests,
278 rondata tests (one ignored, zero filtered), 824 sim tests, 13 fixed tests,
and three doctests, plus install survey, clippy, fmt and paperwork guards.
Rondata took 119.86 seconds. Sampled tree peak was 13,733 MiB under the unchanged
20 GiB ceiling. All 554 fixture requests were present. These single full-suite
measurements are validation evidence, not an isolated causal benchmark.
Log: `/tmp/road-memory-gate.log`; fixture report:
`/tmp/attrition-gate-road-memory-20260910/fixture-coverage.json`.
