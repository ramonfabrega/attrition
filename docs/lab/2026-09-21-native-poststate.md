# Preparing the native path comparison

L65 completes one modeled pathfinding call and matches its internal A* return.
The next question is whether its **resulting path and unit record** agree with
the native call. This follow-up adds an optional observer and an offline
comparator. Its subsequent live result is recorded in
[L67](2026-09-21-native-path-agreement.md). It starts from lab commit `4d16f5c`.

## Capture boundary and contract

`RON_RESTORE_POSTSTATE` extends the existing restore wrapper immediately after
its call to `find_upath`, before the jump back to `0x688faa`. There is no new
original hook site. The existing register-image adapter preserves GPRs,
arithmetic flags and x87/SSE state around the callback; this observer neither
replaces the native return nor writes search state. It is armed once after the
chosen pre-call context succeeds. Python validation additionally requires the
whole pre-call payload to validate: a post-state file alone is insufficient.
The macro requires `RON_RESTORE_PROBE` and `RON_MEMORY_PAYLOAD`, with their
existing graph/census/inventory dependencies. Default builds omit the callback.

The callback checks the frame and the six-argument return-stack boundary,
then captures all 344 unit bytes. It validates owner/id, path length and
capacity, and retains **every capacity slot**, capped at 4,096 slots / 64 KiB.
A zero-capacity path needs no pointer read. Exact `ReadProcessMemory` checks,
32-bit range bounds, and a second whole-unit read reject invalid or changed
inputs. This is not a process-wide atomic snapshot, and the path bytes are not
read twice. Concurrent writes that escape these checks remain a limitation.

The output is a version-1 `restore-poststate.bin`: eight header words, the
exact 216-byte pre-call prefix, nine post-return register words, the 344-byte
unit, and `capacity × 16` path bytes. The fixed prefix is 628 bytes, checked
by the compiler. The header binds frame, unit, prefix length, path length in
bytes, the observed flags mask and caller boundary. INFO 181 records successful
output and native outer EAX; INFO 182 records failure. Short reads/writes and
file-open failure cannot produce an accepted packet.

The decoder requires exact framing, embedded-prefix equality, event identity,
unit identity, stack/flags bounds, complete path capacity and one matching
receipt after the payload and completed native A* calls. The complete existing
payload validator supplies original/tracer image binding and acquisition checks.

## What the comparison will say

`tools/explore/restore_poststate.py INSTALL CAPTURE --replay REPLAY_JSON`
validates the paired native packet and compares against a replay of the **same
payload hash**. It compares outer return EAX, length, capacity, every unit byte
and every path slot. The known unit path-pointer field is reported separately
from other changed bytes. No arbitrary pointer-shaped value is masked.

Active-path agreement and all-capacity agreement are separate fields. An
authored negative control changes only the final inactive byte: active paths
still agree while the all-slot comparison fails and names slot one. Other
controls detect a non-pointer unit change, return mismatch, missing slots and
a different payload. A native allocator may leave different inactive bytes;
those differences will be reported, not erased to obtain a passing result.
This remains a unit/path comparison, not all mutated native state.

## Offline validation and acquisition recipe

The C producer and Python consumer agree byte-for-byte on an authored packet.
C tests cover one-shot behavior, read failures at every acquisition step,
frame/stack/unit mismatches, invalid length/capacity/pointer, changed unit,
empty path and file failures. The shared read helper's existing tests cover
short process reads. Address and undefined-behavior sanitizers pass.
All **116 focused tests** pass with retained-context tests enabled.

Default, congestion/payload and congestion/payload/post-state 32-bit tracer DLLs
compile and link. Both missing-dependency configurations fail compilation as
intended. Initial compilation omitted the already-required census macro;
restoring the complete existing feature set resolves that configuration error.
The full release gate passed at `/tmp/native-poststate-gate`: 332 rondata,
855 sim, 13 fixed and three doc tests; all 782 requested fixtures present;
clippy, formatting, survey and paperwork guards clean. Peak tree RSS was
8,140 MiB with two workers. The final receipt frame check passes its five-test
module, and the shared adapter passes 256 CPU-state cases plus stack and
extended-state mutation controls. Final note edits receive the paperwork guard.

Sources, compiled variants and validation evidence are retained outside git at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-native-poststate`,
with a hash manifest. No live post-state packet is included.

Once a **fresh capture slot** is granted, repeat the retained memory-payload
scenario: map style 14, seed 12345, end frame 1400, timeout 180 seconds, log
window `[18,36)`, `UNITS=3`, `COMMANDMANAGER=1`, and the same retained
`rontrace.cmd`. Use the previous feature set (`RON_CONGESTION_PROBE`,
`RON_SEARCH_CENSUS`, `RON_SEARCH_GRAPH`, `RON_RESTORE_PROBE`,
`RON_MEMORY_INVENTORY`, `RON_MEMORY_PAYLOAD`) plus `RON_RESTORE_POSTSTATE`;
the unattended runner adds `RON_AUTOSTART`. The previous launch-to-exit time
was 22.37 seconds; this is a planning reference, not a promised new duration.

Use a new external output directory and the runner's cooperative lock. Verify
settings restoration and release the lane immediately after the run. Retain
the new sidecar in the external capture manifest, bind the image hashes, then
validate the packet before replay. Newly allocated addresses may differ:
rediscover and verify logical borrowed objects from this packet instead of
copying L65's absolute pointer arguments. Preserve every mismatch as evidence.
No capture was launched during this preparation. The subsequently authorized
run and lane release are recorded in [L67](2026-09-21-native-path-agreement.md).
