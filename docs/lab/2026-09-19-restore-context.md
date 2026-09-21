# Bounded context acquisition for search delegation

The version-2 restore wrapper replay stops at the resumed search's first thread
read. This change prepares acquisition of the missing thread/table inputs at the
same delegation event. Offline validation was followed by one authorized native capture on September
21. Its whole table matches regeneration; no resumed search returned in the
emulator, and no fidelity score moved.

## Acquisition contract

`live_restore_context.h`, included only through the optional restore collector,
writes `restore-context.bin` after a successful prefix write. Its version-1
packet is 792 fixed bytes plus a coordinate allocation capped at 786,432 bytes
(an experimental cap, not a claim about the game's maximum). It contains:

- Frame/unit identity, allocation origin/center, extent and thread pointer.
- The thread's first 28 TIB bytes; self pointer, stack bounds and exception-chain
  head checked against this thread's direct FS reads.
- An exact embedded copy of the 216-byte version-2 prefix packet.
- The adapter's 512-byte FXSAVE image, retained as opaque bytes.
- Every byte of the live coordinate allocation, sized from its origin/center.

The collector checks every memory read and file write for exact completion.
INFO 164 records a successful sidecar write; failures use INFO 163. The reader
requires a unique matching receipt between prefix publication and the following
native A* call, in addition to the existing graph/prefix/trace checks. It checks
source executable, traced executable and tracer hashes against capsule binding.
Missing context is never synthesized. Older captures remain usable through the
existing prefix-only tool.

The table's two-half allocation contract comes from the original initializer
experiment in [L56](2026-09-19-coordinate-table.md). `restore_context.py` runs
that initializer for the captured extent and compares **all table bytes**, not
sampled entries. Original code and captured bytes remain outside git.

## Evidence and limits

The C fixture produces a packet consumed by the Python reader. It exercises 34
failed/short-read cases, write/create failures, inconsistent TIB fields, invalid
table extent, and single-shot behavior. Python tests reject malformed packet
headers, identity, bounds, overlap, truncation, trailing bytes and missing,
duplicate, reordered or mismatched receipts. The authored 48-entry table agrees
with the owned original initializer; changing one byte is rejected.

All 256 emitted-adapter cases now also assert that the collector's FXSAVE address
formula equals the adapter's actual saved-area pointer. Existing register,
arithmetic-flags, stack and extended-state preservation checks still pass.
The Windows-target tracer compiles with all relevant optional probes enabled.

These authored tests establish the acquisition/validation contract; the native
sidecar check below provides a separate live observation. The fixture's FXSAVE bytes deliberately carry no
hardware-state validity claim. The reader does not interpret or load them yet.
The observer samples the TIB during an ordinary C callback with no SEH frame;
whether the full packet is sufficient for downstream replay remains unproven.
A graph and registry header are not a complete unit-state snapshot.

## Authorized native check, September 21

One map-14 run, seed 12345, end frame 1400, using the same 32-spawn congestion
schedule as L57 completed in 27.70 seconds (7.56 build, 20.02 launch to exit).
Exit was zero, 1,401 frames were verified, and all five settings files were
restored. The user was told the capture lane was free immediately afterward.

At frame 224, owner 0 / id 16, the 77,592-byte sidecar passed identity, extent,
thread bounds, exact prefix, receipt ordering and executable/tracer binding.
Its size-400 coordinate allocation is 76,800 bytes: all 19,200 signed entries
(indexes -9,600 through 9,599) match original-initializer regeneration. The
28-byte TIB prefix and opaque 512-byte FXSAVE image were retained. The entry graph
contains 308 records. The wrapper replay still matches in 31 instructions across
256 resets and a fresh runner; the following native A* result is 1.

The scenario records 32 spawned captains, ten orders, 2,314 A* returns and 1,787
suspensions; metadata reaches its configured 64-event cap. These observations do
not establish full-run noninterference or general startup reliability.

Artifacts, settings backup, reports and hash manifest are retained at
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-restore-context`.
Copied files were hash checked and context validation rerun from that directory.
Acquisition scratch is `/tmp/lab-context-20260921/map-14`.

Next: use the captured thread/table inputs in bounded offline replay and measure
the next missing dependency. FXSAVE import remains unimplemented; no complete
search return is claimed. Request a new slot if further acquisition is needed.
The separately paused runtime-isolation experiment stays paused.

## Reproduction

```sh
RON_CONTEXT_IMAGE=/path/to/game/riseofnations.exe \
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_restore_context.py
uv run --offline tools/explore/test_register_image_stub.py
uv run --offline --with unicorn==2.1.4 python tools/explore/test_restore_prefix.py
# Once a fresh, bound capture exists:
uv run --offline tools/explore/restore_context.py /path/to/game /path/to/capture
```

Focused checks pass: five context tests (including the exact C collector), five
prefix tests, 256 adapter states and their negative controls, original-table
comparison and changed-byte rejection, and Windows-target compilation. The full
repository gate passed with two workers: 332 rondata, 855 sim, 13 fixed and three
doctests; 782 fixture requests, none missing. Peak tree RSS was 8,056 MiB under
the 20 GiB cap. Reports: `/tmp/restore-context-gate` and corresponding `.log`.
