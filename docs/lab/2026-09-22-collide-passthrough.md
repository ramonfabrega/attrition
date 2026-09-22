# L84: the collide correction is a carried-through field at this boundary

L82's original caller instruction closes a unit-record gap, but it does not
change the measured pathfinder computation. The L83 compact fixture makes this
distinction inexpensive to test: nine raw 16-bit collide values, each repeated,
produce the same second-call instruction sequence and path. The callee neither
reads nor writes the field on these runs; it survives unchanged.

This qualifies L82 rather than overturning its output comparison. Advancing
collide from 2 to 3 matches the native unit record because the caller owns that
transition. It is not evidence that this field caused a pathfinding difference.
No live capture, main score movement or PR #6 change accompanies this finding.

## Experiment and falsifiers

`tools/explore/collide_sensitivity.py` loads the validated L83 fixture, checks
its complete baseline fingerprint, executes the first call at witnessed limit
95, then changes only the two collide bytes before modeled re-entry at limit
300. Every trial starts from a fresh first-call execution. Values are 0, 1, 2,
3, 4, 255, 32767, 32768 and 65535; these are raw bit patterns, including signed
boundary cases, not a claim that each is a reachable gameplay state.

The second invocation is watched with guest memory and instruction hooks.
Overlapping reads/writes count even when they start outside the two-byte field.
Modeled memcpy/memset accesses execute as guest instructions. The host-side
service argument reader is additionally wrapped during the invocation; it
also records no overlapping reads. Host output observations occur after hooks
are removed, so observing a field does not count as the callee consuming it.
The report preserves this distinction explicitly.

All eighteen invocations return successfully and execute the same 33,494
instruction sequence. There are zero field accesses, including the checked
host-service reads. Each output retains its input collide bits. Every trial
agrees with the native second return, modes, path length/capacity and all 640
path bytes. All unit bytes outside collide are identical across trials,
including the modeled path pointer; the native comparison retains only its
existing explicit path-pointer relocation allowance. Registers, initializedness,
model histories and write fingerprints are identical. The full memory hash is
not expected to match because it includes the deliberately varied field.

Value 3 also passes the complete native boundary comparison. Other values are
compared to that unchanged native witness, not to new native interventions.
All repeat fingerprints agree; the original baseline restores afterward.
The final sweep took 29.37 seconds, including loading, repetitions and reset
checks, with no dependency discovery or broad payload reads.

The observed property is enforced by `require_passthrough`, not just described
in prose. Authored counterexamples reject a field access, changed instruction
sequence, changed registers, another unit difference, altered path bytes,
failed execution, missing trial or non-preserved value. Separate positive
controls execute overlapping guest reads/writes and checked host reads, while
negative controls distinguish nonoverlap and host observations. Hook removal
and service-reader restoration are tested. The affected suite passes 67 tests.
The full release gate passes 1,203 tests, 782 fixture requests with none missing,
clippy, formatting, the install survey and paperwork checks.

## Scope and next direction

This proves non-use only in this selected modeled second callee. It says nothing
about the caller's later use of collide, another saved search, native allocator
behavior, unseen CPU state or whole-world advancement. Nine interventions are
not a global proof about all possible states. Nor is instruction-sequence
agreement alone sufficient; the memory-access controls and output comparisons
are essential parts of this finding.

The earlier caller transition remains correctly observed and implemented as a
bounded original-instruction check. It should not be presented to steering as
an explanation of pathfinding behavior. For a future divergence question, first
separate fields merely carried through the output from inputs actually consumed
by the selected call. This experiment needs no broader caller emulation or new
capture to answer its question.

Reproduce with `PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4
python tools/explore/collide_sensitivity.py INSTALL FIXTURE`, using the L83
fixture directory. Redirect the generated report outside Git. Evidence lives
under `/Users/rf-studio/ron-data/lab-experiments/2026-09-22-collide-passthrough`.
