# From a retained payload to an allocator boundary

One broad capture has now answered nine data dependencies without nine more
capture slots. This is the first practical return from the inventory/payload
experiment. It does not yet establish a faithful full A* resumption.

## What was run

The one-off probe consumes the validated L62 packet and the hash-bound owned
original executable. It starts with the existing narrow replay regions and
checks all overlaps between non-code, non-scratch inputs and the broad packet:
there are no disagreements. Original executable sections supply code; the
return sentinel is excluded. DLL code is not loaded or substituted.

On a missing-data refusal, the probe ends that attempt, reads the containing
page from an explicitly selected payload span, removes overlaps with already
declared regions, and starts a fresh emulator with the additional exact bytes.
Page padding does not grant access. Write permission follows the inventory's
protection. Existing canonical stack/FS inputs and write-before-read scratch
rules remain in force. It is capped at 64 dependency rounds and 10,000
instructions per attempt. The observed execution needed neither cap.

This is **dependency exploration**, not an expanded fidelity test. The captured
FXSAVE image is still not imported. More original code is permitted than in the
previous narrow prefix test. The instruction counts therefore describe this
particular setup and are not the project's fidelity score. No native allocator
has been called, synthesized or bypassed.

## Result

| Newly missing read | Attempted instruction | Result after supplying its captured page |
|---|---:|---|
| World-pointer slot | 91 | Passed |
| Heap input at `0x142b3d54` | 93 | Passed |
| World target at `0xc097e8` | 102 | Passed |
| Heap input at `0x35445828` | 123 | Passed |
| Heap input at `0x35447284` | 134 | Passed |
| Main-image input at `0xb4a140` | 135 | Passed |
| Working-state input at `0xe85ebc` | 249 | Passed |
| Heap input at `0x35446c48` | 323 | Passed |
| Import slot at `0xac54f0` | 382 | Passed; execution then refused the DLL target |

Only **36,256 additional bytes (35.406 KiB)** were declared. Those are page
fragments, not a measured minimal read set. The final refusal is an instruction
fetch at `0x7b950600`, outside the original executable and captured candidate
set. The owned executable's import table identifies slot `0xac54f0` as
`malloc` from `api-ms-win-crt-heap-l1-1-0.dll`. The original call site is
`0x46eb0e`; the emulated stack at refusal contains return address `0x46eb14`
and allocation size **4 bytes**. No allocator result was supplied.

Two independent process runs reproduce the entire dependency sequence, byte
count and terminal refusal. A third records the call arguments and 104 writes
at that boundary. Its write-list SHA-256 is
`c709ab237edf4606f824493c67141f9b480cb5faae2c6ac86997a84d4a4520ed`.
The captures' unchanged native scenario/search projections are a separate
observation; they do not validate these emulated writes or the eventual return.

An additional fresh-process control changes all eight XMM registers, all eight
physical x87 registers, MXCSR and FPCW before each attempt. It produces the same
dependency sequence, allocator arguments and 104-write digest. This narrows an
uncertainty for this prefix only; it does not establish a complete FXSAVE import
or cover later search instructions.

The authored one-off source, three baseline observations and extended-state
control are retained at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-payload-frontier`, with a
manifest. It references the L62 payload hash. This deliberately stays an
exploratory script until its interface and correctness checks earn promotion;
original executable or captured memory bytes have not entered git.

## Next experiment and stopping conditions

The immediate uncertainty is now a runtime service, not another adjacent unit
field. Investigate a bounded **modeled allocation-success path** offline:
explicit call ABI, requested size, an independent scratch arena proven not to
overlap declared captured inputs, uninitialized-byte tracking, and a logged
allocation transcript. No fabricated bytes may silently become captured inputs.
A malloc success model is not the native allocator or its failure behavior;
subsequent realloc/free/import calls must refuse until their contracts are
explicitly modeled and tested. Pointer-sensitive behavior remains a concern.

Before claiming a faithful full search, address extended-state initialization
and compare the native return plus affected output state. A modeled return alone
cannot settle that. The current packet can support those investigations; no
additional capture is needed merely to revisit the nine reads above. Request a
fresh lane only for evidence not already retained.

The repository release gate passed at `/tmp/memory-payload-live-gate`: 332
rondata, 855 sim, 13 fixed and three doc tests; 782 fixtures present. Final
validation notes and the resolved-status amendment receive the paperwork guard.
