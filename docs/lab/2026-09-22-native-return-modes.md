# L81: observe native return modes instead of assuming preservation

The L80 diagnostic rerun identified the refusal: the second wrapper at frame
225 returned 8 with the expected stack displacement, limit 300 and saving 0.
The input saving flag was 1. The collector's assumption that saving remained 1
was wrong for this observed completion. The prior failed captures remain failed;
no missing post-state is reconstructed from the diagnostic values.

## Native evidence and corrected observation

Diagnostic capture:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-second-restore-diagnostics/map-14`.
INFO 193 records actual ESP 2241480, delegation ESP 2241456 and outer return 8;
INFO 194 records successful mode read, limit 300, saving 0, reason 8, frame 225.
This establishes the observed transition, not a general rule for every return.
The run used committed diagnostic collector `1933716`, exited zero in 26.05
seconds and restored all five settings files.

The corrected collector records the two output mode words in a version-3
post-state trailer, without requiring their equality to the input. The second
call still requires input 300/1, no active intervention, a successful mode read,
matching frame/stack/unit, consistent full unit bytes, bounded path capacity and
complete file writes. It never writes the second call's modes. First-call limit
restoration and version-2 provenance remain unchanged.

INFO 195 binds the return-mode words to the unit, output byte count, outer return
and frame. The reader requires exactly one matching receipt before post success,
rejects one on legacy packets, and requires a mode observation on the second
packet. Replay now observes those same two words through its declared-memory
checks; comparison fails if they differ or are absent. Return modes are outputs
to compare, not input values to overwrite or silently ignore.

Fresh corrected capture:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-second-return-modes/map-14`.
Its collector tree is preserved as base `1933716` plus the archived source patch
and hashes. The run exited zero in 21.76 seconds (30.19 total), read back map 14,
seed 12345 and closing frame 1401, restored all five settings files and released
the lane before analysis. Both packet envelopes and complete input validations
pass, including image binding, graph, context, inventory, coordinate table,
payload framing, anchor bytes and receipts. Both provenance-bound projections
retain every native call/census record from the original trace.

The first packet is frame 224, limit 95, outer return -1, path length 1/capacity
10. Its intervention restores limit 300. The second is frame 225, outer return
8, path length 8/capacity 40, and return modes 300/0. Owner 0/id 16 and the unit
address agree across the pair. This identity tuple does not prove object
allocation generation. Neither broad payload is an atomic snapshot.

## Offline replay and validation

The initial second-input replay refuses an undeclared free of its captured
160-byte path array. That is a model boundary, not evidence of a bad native
free. Follow-up extents are derived from this capture's typed headers, never
from a historical run's addresses. The completed replay returns 8 in 34,845 instructions and matches the native
A* return 1. Thirty-nine dependency additions supply 153,740 captured bytes;
three logical borrowed extents total 672 bytes (416 additional to the narrow
graph). They are the ten-slot path and two 64-pointer recycler arrays, derived
from this packet's headers. The intermediate guarded refusals are retained.

The strict native comparison passes: modes 300/0; length 8/capacity 40; all 640
path bytes, including inactive slots; and every unit byte except offsets
0xb8..0xbb, the explicitly reported path allocation relocation. No other unit
byte is masked. Two final-region reset runs reproduce registers, declared
memory, initialization, model state and write fingerprints. The output match is
one captured continuation boundary, not a native allocator or complete-world
equivalence claim.

A separate full-unit comparison between first native post-state and second
native pre-state finds one changed byte: the word at +0x88 advances from 2 to 3.
No meaning is assigned to that word here. The record change itself prevents
treating immediate same-input re-entry as the naturally scheduled boundary.

The affected suite passes 51 tests, including owned-image checks. New controls
exercise short/failed trailer writes, missing/duplicate/mismatched/out-of-order
mode receipts and a mode-only native/replay disagreement. The existing failure
diagnostics and first-intervention restoration controls remain active. The
opt-in Windows tracer passes compilation checks; each live run builds and binds
its actual DLL. The full release gate passed: 1,203 tests, 782 fixture requests with none
missing, plus clippy, formatting, install survey and paperwork checks. Final
receipt-ordering controls reject mode observations before native return; the
51-test affected suite and actual native agreement assertion pass on the final
reader implementation.

The archive is
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-native-return-modes`.
No main score changes. PR #6 stays frozen. Native world-transition modeling,
FXSAVE import and complete-state fidelity remain outside this boundary result.

## Lane agreement

Ramon authorized future bounded attempts through the shared nonblocking capture
lock without a manual slot question for each attempt. The lab and main checkout
use the same profile `.attrition-capture.lock` with exclusive nonblocking flock.
A held lock is left intact; it is never unlinked or bypassed. The harness still
checks that the game is closed before staging, restores settings and releases
the lock on completion. This coordinates compatible runners, not arbitrary GUI
launches. Report lane release promptly; ask when coordination or permissions
actually need human intervention.
