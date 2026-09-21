# Restore collector: ABI migration and a fresh native packet

## Outcome and lane boundary

The optional restore collector now uses an individually saved register image,
arithmetic-flag preservation and an aligned FXSAVE/FXRSTOR area. Its exact emitted
adapter passes 256 authored machine states, including deliberate observer
clobbers of volatile GPRs, x87 and SSE state. Its new version-2 packet retains
the unit-registry header that the previous packet discarded. A fresh native
capture validates that packet, and the original restore wrapper replays against
its recorded delegation state outside Wine.

The main loop is online. The user explicitly granted a free capture window while
Fable was steering and no workers were out. **One bounded Great Lakes capture**
used that window, after the full gate finished. All five backed-up profile files
were restored and verified, the game exited normally, and the lane was released.
Instrument binaries were staged outside the install. The work starts at `9f464f6`
on `codex/jev-lab`; no simulation score changes and no additional Jev calls or cost.

## Register image and preservation contract

`tools/explore/register_image_stub.h` emits a 77-byte adapter. It uses individual
push/pop instructions and LAHF/SETO/SAHF, with no PUSHAD, POPAD, PUSHFD or POPFD.
The image exposes EDI, ESI, EBP, entry ESP minus four, EBX, EDX, ECX and EAX,
followed by a **packed arithmetic-flags word**, not complete EFLAGS. The copy
helper normalizes ESP and converts the last word to the observed mask `0x8d5`
(CF, PF, AF, ZF, SF and OF).

The callback is cdecl: it must preserve nonvolatile GPRs and control flags, with
the ordinary clear-direction-flag entry contract. The adapter restores the
arithmetic flags the callback may clobber. Control flags are preserved under
that contract but are not claimed as captured evidence.

The adapter also reserves and aligns private stack storage for FXSAVE/FXRSTOR.
This preserves x87 and SSE state across the observer rather than relying on C
code to avoid those registers. It assumes the target's existing FXSR/SSE support;
it is not an AVX or general extended-state wrapper. A copy of the register-image
pointer and save-area pointer lives in nonvolatile registers across the callback.
The stack is aligned to 16 bytes at the call site after its argument is placed.

Only the restore-specific optional wrapper is migrated here. Other optional
collectors are not silently covered. The second restore trampoline now begins
256 bytes into its allocation, leaving room for the larger adapter and the
existing displaced instructions/call continuation. No new return replacement,
allocator adapter or search-state write was introduced.

## Executable ABI checks

`test_register_image_stub.py` compiles the same C emitter used by the collector,
then executes its output in Unicorn on 256 deterministic authored states. These
cover all 64 arithmetic-flag combinations, both EFLAGS.ID values, four incoming
stack alignments and varying x87 stack TOP. GPRs, all eight XMM registers and all
eight physical x87 registers have seeded values. The observer model changes
volatile GPRs, arithmetic flags, XMM/x87 registers, tags, TOP, MXCSR and the x87
control word while obeying the cdecl nonvolatile/control-flag contract.

Assertions verify the image passed to the observer, arithmetic flags it records,
all listed state after return, callback alignment and unchanged caller stack.
Every emulated data access must remain within the adapter's private 640-byte
stack allowance. This bound is a test allowance, not a measured native stack
high-water claim; callback internals are modeled separately by the host fixture.

Two deliberate adapter mutations fail: a wrong saved-ESP offset and removal of
FXRSTOR. The decoded-instruction guard is also made to reject each of the four
forbidden bulk-save/restore opcodes. These checks prove the authored cases in
the pinned emulator. They do not prove native Wine startup reliability, arbitrary
callbacks, exception unwinding through the wrapper or instrument noninterference.

## Packet version 2 and backward compatibility

The restore packet grows from 196 to 216 bytes. The original payload order is
retained, followed by an explicit flags mask and four captured registry-header
words. The collector now retains the checked read at
`0xc0aeb4 + owner*0x1c`; word three is the registry pointer whose later search
read was the [coordinate-table experiment's next refusal](2026-09-19-coordinate-table.md).
The already-retained unit address is the result of reading that registry's ID
slot. These are authored packet fields, not a new claim about game struct layout.

`restore_prefix.py` reads both versions. Version 1 retains its full-register
comparison. Version 2 rejects unexpected flags masks, bits outside the observed
mask and invalid registry bounds. Replay compares all eight GPRs and only the
recorded arithmetic flags. Architectural EFLAGS bit 1 is supplied to the emulator;
unobserved control bits are never treated as measurements. Reports expose the
packet version and flags mask. Hook and write receipts must match both the
version and byte length; a new packet with old receipts is rejected.

The exact C collector fixture passes successful collection, all 24 read-failure
and short-read cases, identity bounds, graph failure, failed/short writes and
single-shot behavior. All 64 arithmetic-flag encodings pass through the C copy
helper. An actual 216-byte file written by that fixture is decoded by the Python
reader, checking the C/Python boundary rather than only independent hand-built
fixtures. Five Python tests cover both packet versions, receipt binding,
corruption, registry bounds and partial-flag comparison. An arithmetic-bit or
GPR change fails; an unobserved control-bit change is correctly outside the claim.

The complete optional tracer also compiles for the existing 32-bit Windows target
with `-Werror` and all restore/congestion/graph defines. The offline preflight
wrote only `/tmp/lab-restore-v2-tracer.obj`, without using shared build outputs.
The authorized native acquisition below used the normal capture runner's staged
build during the granted window with no workers out.

## Native acquisition and replay

The seed-12345/map-14 congestion scenario completed frames 0–1400 with 32 spawned
captains, ten group orders, 2,314 A* returns and 1,787 suspensions. The census
reached its documented 64-event cap; first suspension was frame 223. Build took
7.93 seconds and launch-to-exit 20.11 seconds. One launch was attempted, one
completed, with exit code zero and verified map, seed and closing frame 1401.
These are this run's observations, not a repeated-cohort reliability estimate or
a direct noninterference comparison against an uninstrumented run.

The first restore was frame **224, owner 0, ID 16**. Its version-2 packet and
graph pass exact receipts and executable/tracer hash binding. The graph contains
308 records, 223 tree nodes, 67 owned PathNodes and three CollBlocks. The actual
original wrapper replays in **31 instructions**, matching all eight GPRs, the six
observed arithmetic flags, the full delegation stack and declared semantic bytes.
It matches on 256 resets and a fresh emulator. All ten omitted dependencies and
four corrupted outputs refuse as intended. Repath count is 1, delegated limit
300 and saving is enabled. The trace records the following native A* return as 1;
that downstream call is **not** replayed by this wrapper check.

Entering the larger search with the real arguments and graph still refuses on
its fifth instruction at the uncaptured FS:[0] read. This validates the existing
boundary on fresh evidence; a successful wrapper is not a successful resumed A*.

Unlike the lost historical temporary packet, this capture is retained durably at
`/Users/rf-studio/ron-data/lab-captures/2026-09-19-restore-v2`. It contains the
packets, trace, logs, image binding, original/traced executables, tracer, receipts,
settings backup, replay/scenario reports and a file-hash retention manifest.
Copied bytes were checked and image-bound replay rerun from that location. All
owned-install artifacts stay outside git. Temporary acquisition directory:
`/tmp/lab-restore-v2-native/map-14`.

## Remaining offline work before another capture slot

The bounded, versioned thread/coordinate input packet is now implemented and
[validated offline and in one live capture](2026-09-19-restore-context.md);
full downstream replay remains open.
The required inputs are:
the relevant thread state, live table origin/center, extent checks and a live
table comparison against regeneration. Preserving SIMD state across the observer
is not the same as recording it for replay. The existing graph likewise does not
become a complete unit-state snapshot merely because its registry is now retained.

Once that acquisition contract is implemented and tested offline, request another
capture window. Every live check must validate packet/receipt/image binding and
settings restoration. This one successful run does not establish general startup
reliability, full state fidelity or a resumed-search result. The separately paused
runtime isolation experiment stays paused.

## Reproduction and validation

```sh
uv run --offline tools/explore/test_register_image_stub.py
uv run --offline --with unicorn==2.1.4 python tools/explore/test_restore_prefix.py
cc -Wall -Wextra -Werror -Wno-pointer-to-int-cast -Wno-int-to-pointer-cast \
  tools/explore/test_live_restore_probe.c -o /tmp/lab-restore-v2-test
/tmp/lab-restore-v2-test /tmp/lab-restore-v2-fixture.bin
```

The ABI sweep, negative controls, C fixture, cross-language packet check, Python
packet tests, Windows-target compile and fresh capture/replay pass. The full gate
passed with two workers: 332 rondata tests (two ignored), 855 sim tests, 13 fixed
tests and three doctests; 782 fixture requests, none missing. Peak monitored tree
RSS was 8,322 MiB under the 20 GiB cap. Formatting, Clippy, data survey and paperwork
checks passed. Reports: `/tmp/restore-collector-abi-gate`; corresponding `.log`.
