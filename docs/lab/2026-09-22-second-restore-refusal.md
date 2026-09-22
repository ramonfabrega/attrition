# L80: native second input captured; return observation refused

The L79 collector ran from `088738227b806650226afdc67a7d6a32b49737ba` after
Ramon granted the lane. The earlier attempt found the advisory lock held and
made no changes. The authorized retry completed normally: 24.58 seconds from
launch to exit, 32.60 total, map 14 and seed 12345 read back, closing frame 1401,
all five profile settings restored. The harness exited and released the lock;
Ramon was notified before analysis. This is a collector experiment, not a main
score movement or a successful second-call fidelity check.

## Retained evidence

Capture directory:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-native-second-restore/map-14`.
The actual images are bound in `capsule-image.json`; the original image retains
SHA-256 `30478a44b577cb11ebcbbbf53d3e93ba02fd2aacf3bdefa6552c9b6449625079`.

The selected identity is owner 0, id 16, at the same unit address in both
packets. The first wrapper is frame 224; the next selected wrapper is frame
225. Both prefixes, saved graphs, contexts, inventories and payload files exist.
Their payload success receipts report 839,176,192 and 839,372,800 bytes, each
171 ranges, taking 1,369 and 1,389 ms respectively. These receipts establish
collector completion, not independent replay validation or atomic snapshots.

The first post-state is 820 bytes, outer return -1, with path capacity ten.
Its intervention receipts witness the limit change 300 to 95 and restoration
from 95 to 300. The second A* proxy returns 1; that inner result is not the
outer wrapper result. The second post observer emits INFO 182 reason 1, followed
by INFO 192 reason 9, and produces **no second post-state file**. The strict
paired reader refuses this capture with `collector failure in paired trace`.
No incomplete packet was promoted into a usable pair, and no native second
output comparison or replay agreement is claimed.

## What the refusal does and does not establish

The old reason 1 covers intervention-state checks, the second return's mode
read/value checks, callback frame, and expected stack displacement. It does not
identify which check failed. A read of the wrapper alone cannot settle runtime
values after its larger callee executes. Removing one check or changing its
expected value would therefore be speculative. The capture is retained intact.

The follow-up observer keeps the same acceptance predicates and adds failure-only
receipts. INFO 193 records packet index, unit, actual callback ESP, delegation
ESP and outer return; INFO 194 records packet index, precise refusal reason,
mode-read success, limit and saving values. Their final trace word is the actual
callback frame. No additional game state is written. Failed mode reads remain
explicitly invalid even if the zero-initialized observation buffer contains data.

Reasons 6 through 11 distinguish intervention-state failure, failed mode read,
mode value mismatch, frame mismatch, stack arithmetic overflow and stack
mismatch. Existing identity, capacity, consistency and file failures retain
reasons 2 through 5. These diagnostics cannot substitute for a post-state: the
paired reader refuses either diagnostic tag even without the older failure tags.

## Validation and next experiment

Authored controls force mode drift, saving drift, failed reads, frame drift and
stack mismatch and check the exact emitted diagnostics. They also assert that
no second post file was written and the drifted mode was not corrected. Existing
first-intervention restoration and callback failure controls remain in force.
The affected collector suite passes 50 tests, including the owned-image checks.
The full release gate passed: 1,203 tests; 782 fixture requests with none
missing; clippy, formatting, install survey and paperwork checks. The diagnostic
tracer also compiles and links as COFF i386 in a private staging directory.

Both retained wrapper prefixes reproduce their recorded delegation registers,
stack and modes through 16 reset executions and a fresh engine. Each rejects
ten omitted dependencies and four corrupted outputs. This checks the prefix
only, ending before the larger callee; it does not bypass paired-payload
validation or establish the second return values.

The next useful live run is the same bounded scenario with these diagnostics,
on a fresh lane grant. The current slot has been released. Only after two valid
post packets exist should the packet projection/replay comparison proceed.
The report and offline assessment are archived under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-second-restore-refusal`.
PR #6 remains frozen. FXSAVE import, broad-snapshot atomicity and modeled world
transitions remain separate limits.
