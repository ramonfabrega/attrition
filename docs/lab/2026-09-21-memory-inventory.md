# Bounded address-space inventory at search delegation

The next experiment from L60 is implemented as an opt-in metadata collector.
It has offline checks, compile/link validation and one validated live enumeration.
It copies **no memory payload**, suspends no thread, and calls no dump library.
A complete inventory is not an atomic process snapshot or a complete replay input.

## Acquisition contract

`RON_MEMORY_INVENTORY`, with the existing restore/context/graph probes, calls
`capture_memory_inventory` after context receipt 164 and before the native A*
call. The collector records the guest's reported application-address bounds and
page size, then walks `VirtualQueryEx` ranges through those bounds. Every query
must return a complete seven-DWORD record and advance the cursor without a gap,
overlap, invalid state, misalignment or wrap. It retains free and reserved
ranges too, rather than disguising exclusions as absent metadata.

Bounds: at most 8,192 records and 2,000 GetTickCount milliseconds of enumeration.
The time check occurs between synchronous calls; it does not preempt a blocked
single OS call. The scenario runner's timeout is separate. Successful completion
requires reaching the declared end within both limits. On failure a partial
metadata file may remain, but its status and INFO 166 prohibit treating it as a
complete inventory. Checked file-write failure also emits INFO 166.

The version-1 wire format is 280 fixed bytes plus 28 per range. The 64-byte header
contains identity, bounds, count/status/cursor, timing, main image base, an address
inside the observer, and both caps. An exact 216-byte restore-prefix copy binds
it to the context event. Each range records base, allocation base, allocation
protection, size, state, protection and type. Compile-time size assertions guard
the wire format. INFO 165 publishes exact successful size/count/timing; validation
requires it after the context receipt and before the same native A* call, with
no inventory failure receipt.

## Source of the platform layouts

The Win32 target uses the explicit seven-DWORD MEMORY_BASIC_INFORMATION32 layout
published by Microsoft. SYSTEM_INFO is represented by nine DWORDs on this
32-bit target: its first union is one DWORD and its pointer-sized members are
four bytes. Its application-address minimum, maximum and page-size fields bound
the walk. The collector requires the guest's reported processor architecture to
be x86. This is not an assertion about the host's physical architecture.
[MEMORY_BASIC_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-memory_basic_information),
[SYSTEM_INFO](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-system_info).

`VirtualQueryEx` groups consecutive pages by their allocation, state and access
attributes. Free-page allocation/protection/type fields and reserved-page
protection are undefined; classification must test state before those fields.
[VirtualQueryEx](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualqueryex).

## Offline interpretation

`memory_inventory.py` validates the inventory against the existing prefix/context
and image binding. It counts categories separately: free/reserved, guards,
unreadable pages, observer image and active-stack ranges, executable pages,
main-image data, private data, other images, mapped data and unknown types.
The observer's allocation must be a distinct committed image.

The initial candidate set is readable, non-executable main-image data plus
private data after observer-image/active-stack exclusions. This is an explicit
experimental selection, not a claim that private memory is simulation state.
It may include asset caches and other threads' stacks, or exclude something the
search later needs. The report compares candidate size against 128/256/512/1024
MiB budgets and lists its largest allocations and known-root ranges. A budget
comparison does not authorize or perform a future payload copy.

## Tests, build boundary and reproduction

The authored C fixture exercises 14 ranges, 28 failed/short queries, malformed
ranges/system bounds, count/deadline exhaustion, clock wrap, file creation/write
failure and short writes. Python consumes that exact C output, checks coverage
and candidate accounting, rejects bad headers/ranges/receipts, ignores undefined
free-page fields, and refuses a non-distinct observer image. Six tests pass.
Seven context tests pass. Both control and inventory-enabled Windows tracer
builds compile and link; only the latter imports the new APIs.

Three names are added to `tools/trace/kernel32.def`: VirtualQueryEx, GetSystemInfo
and GetTickCount. They are only referenced under the optional probe define, so
normal builds acquire no new imports. This shared build-surface addition is a
separate adoption concern from the lab collector and must be kept distinct from
any request to change the main commander's behavior.

```sh
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_memory_inventory.py
# After an authorized capture with RON_MEMORY_INVENTORY:
uv run --offline tools/explore/memory_inventory.py /path/to/game /path/to/capture
```

The repository release gate passed on 2026-09-21 with two test workers:
332 rondata, 855 sim, 13 fixed and three doc tests; all 782 requested fixtures
were present. Clippy, formatting, data survey and paperwork checks passed.
The gate report is `/tmp/memory-inventory-gate`; its measured peak was 8,295 MiB.
The 256-case register-adapter check also passed.

## Live measurement

After the user granted a fresh lane, one run completed in 29.35 seconds
(8.22 build, 21.00 launch to exit), exited zero, reached closing frame 1401,
and restored all five backed-up settings. The lane was released immediately.
At frame 224 the inventory covered `[0x10000, 0x7fff0000)` in 873 records,
24,724 bytes. GetTickCount reported zero elapsed milliseconds: that is the
clock's observed resolution, not proof the walk took no time. The complete
receipt, extent, prefix binding and event ordering validate.

| Category | Bytes | Meaning |
|---|---:|---|
| Private readable non-executable data | 829,145,088 | Candidate, may include assets and other stacks |
| Main-image readable non-executable data | 5,181,440 | Candidate, includes the world-pointer slot |
| Combined candidate | 834,326,528 | 795.676 MiB; exceeds 512 MiB, fits 1 GiB |
| Observer image | 9,568,256 | Excluded |
| Active stack | 1,040,384 | Excluded; canonical delegation stack retained separately |

This measures readable address extents, not physical memory residency or
compressed output size. The largest candidate allocation is 27,197,440 bytes;
29 private allocations each contribute 16 MiB. Their sizes do not identify
what they contain. The unit and unit-type addresses share one allocation;
the coordinate table occupies another. The readable non-executable data in
these two allocations plus the main image totals 38,342,656 bytes (36.566 MiB).
That is a useful lower-cost selection, **not** a complete search-state closure:
the value of the world pointer and its target allocation are still unknown.

The scenario/search projections match the preceding unit-context run exactly.
The new packet reproduces the same 50-instruction A* entry and 91st-instruction
world-pointer refusal at `0xc06188`; 64 resets, fresh-instance comparison and
extended-state perturbation checks pass. No full search returned, and no
fidelity score moved. No new process-memory payload was copied by the inventory;
the existing bounded context/graph probes still produced their usual packets.

Artifacts, including original binaries, remain outside git at
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-memory-inventory`.
Its retention manifest hashes 26 files, including the settings backups and the
derived known-root selection report.
Validation of the retained copy reproduces the inventory report exactly.

## Consequence for the next experiment

The first small selection is now an offline comparison, implemented in
`tools/explore/allocation_selection.py`. It consumes validated inventory rows
and explicit roots, deduplicates their allocations, excludes executable,
guarded, observer and active-stack spans, and refuses absent/ineligible roots,
malformed selected ranges or a byte-cap overrun. Five authored tests pass; the
live known-root selection reproduces 38,342,656 bytes across nine spans in three
allocations. It does not resolve the world pointer or read live memory.

A 128 MiB selected packet would be cheaper but could repeat the dependency chase
this experiment is meant to avoid. The broad candidate fits 1 GiB, so the next
prototype should instead evaluate a streamed broad copy with a hard 1 GiB cap,
a separate copy-time limit and bounded working buffers. Retain the small policy
as a comparison, then use offline replay's measured read set to reduce a broad
packet. The inventory has measured extents only: copy time, coherence and actual
replay sufficiency remain unknown. A cap overrun must refuse rather than silently
shrink the packet, and excluded mapped/DLL/executable regions remain explicit.

Before another capture, the collector needs offline tests for cap arithmetic,
chunk boundaries, exact reads/writes, partial-file failure and time exhaustion.
Native result/output capture and repeated anchor checks remain necessary to
judge replay usefulness and detect drift. This is the next lab experiment, not
a main-loop adoption request or a claim of atomic acquisition. The user was told
to release the capture lane; a fresh slot will be requested when the collector
is built and tested.

The post-capture release gate passed at `/tmp/memory-inventory-live-gate`, with
all 782 fixtures available; six inventory and five allocation-selection tests
also pass. Final report/ledger amendments receive the paperwork guard separately.
