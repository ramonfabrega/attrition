# Regenerating the coordinate table before resumed search

## Outcome

The missing `div_3_table` dependency is reproducible from a scalar extent under
an explicit successful-allocation contract. Executing the original initializer
checked **75,456 entries across six sizes**, all equal to mathematical floor
division by three, including negative indices. Supplying the generated table
and its pointer moves the modeled search prefix from instruction **14 to 22**,
where it refuses the owner-zero unit registry. The search has loaded the table
pointer but has not consumed a table entry before this refusal.

This continues [the thread-state experiment](2026-09-19-jev-resumption.md), on
`codex/jev-lab` from `7de8aad`. It changes no simulator, main-loop document, game
settings or fidelity score. No Jev requests were made; estimated cumulative API
spend remains $0.059800. No fresh live capture was attempted in this step.

## Establishing the initializer rather than guessing the table

The owned `sbl/rise_z.map` names `init_coord_lookup_array` at `0x681db0`, its
allocation-origin slot at `0xcab3ac`, and the shifted table pointer at `0xcae5fc`.
The source image and map hashes are those recorded in the preceding report.
The decompile is useful navigation, but its displayed stack-argument signature
is misleading: the listing and executable take the size from **ECX**. The probe
supplies it there and checks what the original actually requests and writes.

For the tested positive size `n`, the original requests `192*n` allocation bytes,
places the indexed pointer halfway through that allocation, and initializes
`48*n` signed 32-bit entries. Their index range is `[-24*n, 24*n)`. The comparison
specification is `table[i] = floor(i/3)`, independently expressed with Python
integer floor division. No decompiled function was transcribed into the tool.

The menu caller loads 400 into ECX before the call at `0x595c36`. The rules-data
caller at `0x58f34a` instead loads a runtime field and scales it by four. Those
are listing observations, not a claim that a particular live game's extent is
400. A future capture must establish the live extent rather than borrowing the
menu value. Allocation failure, overflow, zero and negative input behavior are
outside this experiment; the harness accepts only positive sizes through 4096.

## Executable contract

`tools/explore/coord_table_probe.py` executes two bounded phases, loading code
from the owned install at runtime:

1. Execute the initializer up to its indirect allocation call. Its import slot
   points to a stop address with **no stub code**. Check the requested byte count,
   actual return address and saved stack.
2. Model the allocator returning an exact-sized fresh allocation, resume at the
   original continuation, and run to the original return. Preserve nonvolatile
   registers and deliberately clobber ECX/EDX to avoid depending on the allocator
   preserving volatile values.

The allocation begins as uninitialized scratch: every emulated read requires a
preceding write, and the checker also requires every output byte to have been
written. The origin and center outputs have the same rule. Callee-saved registers,
stack balance, both pointers and **every table entry** are checked. The allocator
and its failure path are not executed or claimed as verified; the successful
allocation contract is explicit. Original code and generated table bytes stay
outside the repository. The checked-in [observations](2026-09-19-coordinate-table-observations.json)
contain hashes, dimensions, counts and refusal locations only.

| Size input | Entries checked | Allocation bytes |
|---:|---:|---:|
| 1 | 48 | 192 |
| 2 | 96 | 384 |
| 17 | 816 | 3,264 |
| 128 | 6,144 | 24,576 |
| 400 | 19,200 | 76,800 |
| 1,024 | 49,152 | 196,608 |

The total is across overlapping domains, not 75,456 unique indices. Each run
checks its whole allocation. Six negative controls fail as intended: missing
allocation import, allocation shortened by four bytes, changed negative entry,
changed last entry, corrupted center pointer and missing write evidence.

## The next refusal and acquisition requirements

The existing thread probe now optionally accepts the generated table. With the
size-1024 result, the modeled search refuses a four-byte read at `0xc0aec0`, PC
`0x682f77`, on attempted instruction 22. Its earlier thread and table-pointer
refusals remain independently reproducible. Null memory is still unmapped.

This is the owner-zero registry-pointer slot the existing restore collector
already reads as part of its local validation. That local registry read is not
retained in the old restore packet: the callback retains the selected unit
address, while the registry array is a local. A fresh acquisition needs the
actual slot value and the indexed unit pointer, not a guessed object registry.
The old graph is a partial structural snapshot, not all unit state the resumed
search might read. Further fields should be acquired as execution requests them.

The minimum next acquisition therefore has three obligations:

- Retain image-bound restore/delegation registers, arguments, graph and the
  relevant registry indirection as one event, with frame/owner/unit receipts.
- Record thread-state inputs and the live coordinate allocation origin/center
  so its extent can be validated. Compare regenerated table bytes against the
  live table before treating regeneration as faithful to that capture.
- ~~Migrate and execute-test the optional collector's register-image wrapper
  before launching it.~~ Done in the
  [restore collector ABI experiment](2026-09-19-restore-collector-abi.md), including
  one successful native capture. [The prior migration](2026-09-09-hook-restore-migration.md)
  explicitly left that wrapper using bulk register/flags saves. This pass does
  not reuse it unchanged or restart the paused runtime isolation experiment.

The first two are now concrete input requirements. The third is an acquisition
prerequisite, not evidence that an offline initializer needs Wine. A fresh live
packet remains necessary to claim actual resumed-search replay. Nothing here
establishes a successful A* return, full thread emulation or a fidelity gain.

## Reproduction and validation

```sh
uv run --offline tools/explore/coord_table_probe.py /path/to/game
uv run --offline tools/explore/test_coord_table_probe.py /path/to/game
uv run --offline tools/explore/test_thread_context_probe.py
uv run --offline tools/explore/test_bounded_call.py
```

The six-size original-code sweep, four initializer test methods (including the
six negative controls), eight thread-context tests and eleven existing bounded
tests pass. Missing original input refuses rather than skipping. The full release
gate passed: 332 rondata tests (two ignored), 855 sim tests, 13 fixed tests and
three doctests; 782 fixture requests, none missing. Peak monitored tree RSS was
11,991 MiB under the 20 GiB cap. Formatting, Clippy, data survey and paperwork
checks passed. Reports are in `/tmp/coord-table-release-gate`; the complete log
is `/tmp/coord-table-release-gate.log`.
