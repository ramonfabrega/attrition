# Bounded address-space inventory at search delegation

The next experiment from L60 is implemented as an opt-in metadata collector.
It has offline checks and compile/link validation; live enumeration is pending.
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

Live validation remains pending. The user confirmed Fable is using the capture
lane, so no inventory run was launched. A fresh slot is required before running
the single bounded scenario. No claimed snapshot size exists yet.
