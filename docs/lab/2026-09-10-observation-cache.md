# Reuse bounded owned replay observations

## Measured opportunity

Two leader tests reread the same four sibling captures to recover end-frame
seeds, animation lengths, and figure-clock observations. Temporary size
instrumentation measured their source text and allocated observation vectors:

| Sibling | Source bytes | Allocated observation-vector bytes |
| --- | ---: | ---: |
| run11 | 30,409,192 | 0 |
| run3 | 152,269,808 | 156,096 |
| run12 | 262,222,051 | 305,856 |
| run13 | 642,100,524 | 762,272 |

The total is about 1.09 GB of source for 1.22 MB of these owned products.
Instrumentation was removed; `/tmp/replay-observation-size.log` retains the
measurement. This is not the size of the full Initial or all parsed state.

## Cache contract

The indexed reader caches only those three complete owned observation products,
keyed by canonical path, length, and modification time, matching its existing
index identity contract. Setup is still read and parsed on each call. On a
hit, observations are copied into that caller's Initial; caller mutations do
not affect another read. On a miss the existing bounded frame scan runs, the
source is validated, and an eligible result is retained. Metadata is checked
again before yielding an Initial, including on hits.

The process-wide observation cache has an 8 MiB accounted-storage budget and
32-entry FIFO limit. Accounting includes vector capacities recursively through
frame units and their Guy arrays, entry structs and paths. Allocator overhead,
Arc headers, transient references, and caller-owned copies are outside that
accounting; this is not an 8 MiB RSS ceiling. Oversized products are not cached.
The existing offset cache remains separate. No raw capture text, borrowed
strings, or constructed simulation is retained, and no dependency is added.

Cache locks are not held during file I/O, parsing, or callbacks. Concurrent
cold misses may compute the same product more than once; this implementation
does not promise one scan per file. A poisoned cache is bypassed. As with the
index, same-length edits that also preserve modification time are outside the
metadata identity guarantee; use finalized captures. This is not content-hash
binding or an atomic file snapshot.

## Checks with teeth

Disabling budget-based eviction deliberately failed the eviction regression:
`/tmp/observation-cache-bound-negative.log`. Restored tests cover oversize
rejection, oldest-entry eviction, path replacement, the entry ceiling, nested
caller-mutation isolation, preserved duplicate seed ordering, and path/length/
mtime mismatches. The indexed setup tests now exercise both cold and warm
reads and mutate the returned observations. A changed source refuses even
when observations were previously cached.

The retained-corpus equality test now warms the sibling cache before comparing
every Initial field with whole-text parsing, excluding only the already
omitted audit frame bodies. It adds four observed fixture lookups, not new
capture dependencies. Twelve indexed/cache tests pass; the complete warmed
corpus comparison passes separately. Logs:
`/tmp/observation-cache-focused-final.log` and
`/tmp/observation-cache-warm-corpus-equality.log`.

## Isolated timing

Clean release binaries ran the same two leader tests in fresh processes, one
test thread, explicit RON_INSTALL, and before/after/after/before order. The
second test in each candidate process can reuse the first test's observations.
Compilation and instrumentation are excluded.

| Sample | Version | Peak RSS, bytes | Test seconds | Process wall seconds |
| --- | --- | ---: | ---: | ---: |
| 1 | Before | 957,808,640 | 5.17 | 5.52 |
| 2 | After | 993,198,080 | 4.02 | 4.32 |
| 3 | After | 992,673,792 | 4.09 | 4.09 |
| 4 | Before | 873,316,352 | 5.16 | 5.17 |

This workload improves by about 21–22%, with higher measured RSS in the
candidate samples. It is a speed/retention tradeoff, not a memory reduction
or universal full-suite speedup. Logs:
`/tmp/observation-cache-{1-before,2-after,3-after,4-before}.log`.
No gameplay behavior or fidelity score changes; the runtime experiment stays
paused. Broader migration of whole-text sibling consumers remains separate.

## Full gate

The monitored four-thread gate passed with the explicit install: 34 Python
tests, 278 rondata tests (one ignored, zero filtered), 824 sim tests, 13 fixed
tests, and three doctests; install survey, clippy, fmt, and paperwork guards
also passed. The rondata suite took 119.09 seconds. Sampled peak tree RSS was
14,145 MiB under the 20 GiB ceiling. This does not establish a whole-suite
speedup over the previous 117.51-second gate. The fixture audit observed
554 requests with zero missing fixtures; the four added requests are the
explicit sibling-cache warmup.

Evidence: `/tmp/observation-cache-final-gate.log` and
`/tmp/attrition-gate-observation-cache-20260910/fixture-coverage.json`.
