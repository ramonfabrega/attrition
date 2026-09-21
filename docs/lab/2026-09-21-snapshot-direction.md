# Proposed next experiment: one bounded memory inventory, then selective replay

The measured chain is now 5 → 14 → 22 → 50 → 73 → 91 attempted instructions,
with different declared-input stages behind those counts (not a fidelity score).
The first four dependencies were thread/table/registry state, then the current
unit, and now the world pointer. The captured graph is internally closed but is
not the process state the algorithm consumes. Another four-byte patch will not
settle whether a complete search is cheaply replayable.

The next question should be acquisition cost: can we retain enough state once to
resolve multiple future reads offline? The proposal is a bounded inventory first,
with a conservative byte budget and explicit omissions. The [metadata-only inventory collector](2026-09-21-memory-inventory.md) is now
validated live: 873 ranges and a 795.676 MiB broad candidate set. Known-root
allocations plus main-image data cover 36.566 MiB; the world target remains
unknown. A 128 MiB allocation-selection policy is tested offline as a comparison.
Because that can repeat the missing-dependency capture cycle, the next prototype
is a streamed broad candidate copy capped at 1 GiB, with a separate time limit
and explicit exclusions. Payload capture remains unimplemented.

## Comparison

| Candidate | Advantage | Cost or unanswered question |
|---|---|---|
| Add World + UnitType roots | Small, typed inputs; PDB gives extents 0x174 and 0x5d8 | Arrays/pointees remain absent; further capture rounds likely |
| Dump all accessible process memory | Fewer guessed dependencies | Large input, observer-modified stack, concurrent-thread coherence, assets and DLL state |
| Inventory ranges, then retain bounded selected data | Measures size before choosing; explicit provenance and readable-range checks | New collector/reader required; still not an atomic process snapshot |
| Generic minidump inside the hook | Existing container format | In-process deadlock/stack caveats and a new DbgHelp dependency |

I favor the inventory-led experiment. Microsoft documents VirtualQueryEx as a
range query grouped by state/allocation/protection, not an atomic snapshot:
[VirtualQueryEx](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualqueryex).
ReadProcessMemory requires the whole requested area to be readable and returns
the transferred length, supporting exact-read validation:
[ReadProcessMemory](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory).
Microsoft recommends a separate process for MiniDumpWriteDump where possible and
warns that the calling thread's stack trace may be invalid:
[MiniDumpWriteDump](https://learn.microsoft.com/en-us/windows/win32/api/minidumpapiset/nf-minidumpapiset-minidumpwritedump).
Those API facts do not establish Wine behavior; that remains a measured question.

## Concrete experiment boundary

1. Reuse the established delegation event and source/tracer/frame/unit binding.
   Inventory committed accessible ranges with their full type/protection and
   allocation metadata. Preserve excluded regions and reasons too. Require an
   explicit address-space ceiling and a maximum record count; no silent truncation.
2. Separate the game's original image data, candidate private data, other image
   and mapped regions, and observer-owned memory. A protection flag alone cannot
   say whether a private allocation is simulation state or an asset cache.
3. Before copying, sum candidate bytes. A fixed experimental cap may refuse the
   copy, but still leaves a labeled inventory. Do not call a capped inventory a
   complete snapshot or zero-fill missing pages. Do not suspend arbitrary other
   threads or replace the game runtime to make acquisition work.
4. If a later copy is authorized, stream checked chunks to an external artifact;
   require exact reads/writes and a completion receipt. Repeat small anchor reads
   around the pass to detect obvious drift, without calling that proof of global
   coherence. Retain the canonical delegation registers/stack separately from
   the observer's own stack image.
5. Offline, load only captured declared bytes. Immutable original code still
   comes from the hash-bound owned executable; copied DLL addresses do not
   authorize arbitrary imported execution. Record the actual read set and writes,
   keep missing-input failures, and derive a smaller replay packet from that set.
6. Judge usefulness by missing-input boundaries closed per capture, bytes/time,
   repeated-reset cost, and eventually agreement with a native result and its
   observed output state. Agreement on a return value alone is insufficient.

This is a direction choice within the lab, not an adoption request for the main
loop. Existing narrow packets remain valuable reference cases. The captured v2
case stays pinned at the world-pointer refusal while this approach is evaluated.
