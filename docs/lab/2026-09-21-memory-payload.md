# A bounded streaming memory payload, validated on one live run

L61 measured 795.676 MiB of broad candidate data. Selecting only the known-root
allocations reduced that to 36.566 MiB but left unknown dependencies. This
experiment prepares one broader acquisition so future missing reads can be
investigated offline. It does not establish that a full search can be replayed.

## Implemented boundary

`RON_MEMORY_PAYLOAD` requires the existing restore/context/graph probes and
`RON_MEMORY_INVENTORY`. After a successful inventory receipt, at the same
restore delegation event, it selects committed readable non-executable private
data and main-image data. Guards, inaccessible pages, other images, mapped
regions, the observer image allocation and ranges overlapping the active stack
are excluded. The complete inventory is embedded, retaining the exclusions.
No excluded range is synthesized or zero-filled.

The payload cap is **1 GiB**, checked before any payload file is created. A
**1 MiB static buffer** streams checked reads and writes; it is in the excluded
observer image. The copy has a separate **5,000 ms GetTickCount budget**, including
selection, anchor reads, metadata, copying and cleanup. Checks occur between
synchronous calls and cannot preempt one blocked OS call; the unattended runner
still has its independent 180-second timeout. Clock wrap is handled by unsigned
subtraction. Timing is at GetTickCount's observed resolution.

Every chosen range's seven-word VirtualQueryEx result must match the inventory
immediately before its copy and in a second pass after all copies. Every
ReadProcessMemory and WriteFile call must succeed and report the exact byte
count. File creation uses CREATE_NEW. CloseHandle must succeed. A failure leaves
at most a partial file and emits INFO 168, never a success receipt. No allocator
adapter, thread suspension, runtime replacement or search-state mutation is used.

## Drift checks and what they cannot establish

The complete captured unit and coordinate table, table origin/center pointers,
and world-pointer slot are anchors. The unit/table/pointers must match the
existing context before copying. The world value is first read at this event.
Every anchor must be covered by the selected ranges. Anchors are compared with
the streamed bytes, including overlaps split across chunks, and read again after
the copy. This catches some changes that before/after reads alone would miss.

It remains **non-atomic**. Other threads can change non-anchor bytes, mappings
can change and return between queries, and anchor values can change outside the
sampled checks. Runtime-owned private allocations and other threads' stacks can
be selected. The observer's stack is deliberately absent; the canonical bounded
delegation stack and register packet remain separate. A future replay loader
must handle provenance/overlap explicitly and refuse missing dependencies;
a successful file does not authorize arbitrary imported code execution.

## Wire format and validation

`memory-payload.bin` is authored framing around original-derived bytes:

1. A 64-byte version-1 header: magic `0x31504d52`, version, frame, unit, selected
   count, payload byte total, cap, time budget, chunk size, inventory extent,
   excluded stack bounds, observer allocation, initial world pointer, two zeros.
2. The exact complete `memory-inventory.bin`, including its prefix binding.
3. In inventory order, a 12-byte `(inventory index, base, size)` descriptor and
   exact payload bytes for every selected range.
4. A 32-byte footer: magic `0x31444e45`, version, count, payload total, elapsed
   time before footer I/O, world pointer, anchor-byte total, zero.

INFO 167 records `(0, unit, payload bytes, range count, final elapsed)` at the
same frame. It must follow INFO 165 and precede the corresponding native A* call.
No INFO 168 is permitted. Footer time must not exceed receipt time; both are
below the time cap. Exact file extent, range order, all policy/header fields,
embedded inventory and anchor bytes are checked by `memory_payload.py`.
It streams at most 1 MiB per body read, computes whole-file and per-range SHA-256,
and reports file offsets for later selective reads. Those hashes are artifact
integrity aids, not evidence of global state coherence. The validator first
runs the existing image, prefix, graph, context and inventory checks.

The framing cap excludes metadata: on the retained L61 inventory it selects
166 ranges and 834,326,528 payload bytes, for a projected 834,353,340-byte file.
That is a policy calculation against an older inventory, **not** a newly copied
payload or a prediction that the next process has identical allocations.

## Offline evidence and remaining experiment

The actual C writer runs against authored virtual memory in
`test_live_memory_payload.c`, and its exact output feeds the Python validator.
Failure/short-transfer controls cover all 15 read and ten write positions;
failed/short/changed queries cover all six mapping positions. Every one of
65 deadline positions refuses. Tests also cover a timer wrap, exact cap versus
overrun, ineligible anchors, changed unit/table anchors before/during/after copy,
file creation, failed close, and cleanup. Each failure prohibits success.

Eight Python tests cover C/Python agreement, bounded streaming, split anchors,
every header word, inventory/descriptor mutations, truncation, footer/extra
bytes, body anchors, byte caps, exclusions and receipt ordering/failure. The authored C checks also pass AddressSanitizer and UndefinedBehaviorSanitizer. Seven
context tests (including the original-table check) and six inventory tests pass.
Control, inventory-only and payload-enabled Windows DLLs compile/link with
`-Werror`. Payload and inventory builds import identical API names; control
imports none of the optional inventory APIs. The isolated build lives under
`/tmp/lab-payload-link` and did not stage or launch the game.

The wire structs moved to shared lab headers without changing their layouts;
the inventory helper now returns success so a failed inventory cannot start a
payload. This adds no shared trace build definitions or main-loop changes.

```sh
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_memory_payload.py
# After a fresh authorized run with RON_MEMORY_PAYLOAD and the existing probes:
uv run --offline tools/explore/memory_payload.py /path/to/game /path/to/capture
```

The first live acquisition and comparison with L61 are recorded below.
The next step is bounded offline replay, measuring actual reads and writes. Native
return/output agreement is still required before calling a full search faithful;
neither the broad packet nor unchanged scenario projections establish that.
No fidelity score has moved. All original-derived payloads stay outside git.

The repository release gate passed with two test workers at
`/tmp/memory-payload-gate`: 332 rondata, 855 sim, 13 fixed and three doc tests;
all 782 fixtures present, clippy/format/data survey/paperwork clean. Peak memory
was 8,553 MiB. The final validation note receives the paperwork guard separately.

## First live result

The user granted one fresh capture slot. The fixed-seed map-14 scenario reached
closing frame 1401 and exited zero. Total elapsed time was 31.14 seconds
(8.62 build, 22.37 launch to exit); all five settings files were restored, and
the user was immediately told the lane was free. No second capture was used.

At frame 224, the inventory contained 875 ranges. The payload selected 166
ranges and copied **834,981,888 bytes (796.301 MiB)**. The file is 835,008,756
bytes including framing and inventory. Footer and final receipt both report
**1,617 ms**, below the 5,000 ms limit. All 77,156 anchor bytes match. The world
pointer slot holds `0xc097e8`. This is one measured run, not a throughput guarantee.

The complete payload validator passes, including its embedded inventory,
completion ordering, image binding, anchor checks and stream hashes. The
scenario/search projections match the inventory-only run exactly. The existing
narrow replay remains pinned at its 91-instruction missing-world refusal, as
expected when it is not given the new bytes.

The payload SHA-256 is
`d0b4ce6ca20fcf2c1e17f6d385d48150cf84660c706ae944c3b372897824756f`.
The retained capture is
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-memory-payload`; its manifest
hashes 27 files, including the settings backups. Validation from the retained
copy produces exactly the same report. All original-derived bytes remain
outside git.

The [first offline use](2026-09-21-payload-frontier.md) resolved nine additional
data boundaries from this packet and stopped at a CRT allocator call. That is
a useful change in the research boundary, not a full-search fidelity result.
