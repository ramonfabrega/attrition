# Preparing the native continuation-limit falsifier

L68 predicts suspension at limit 95 and completion at 96 for the retained
saved search. This landing prepares **one explicitly altered native call at
95**, without yet running it. No capture lane is held, no TypeSafe call is
made, and no main score changes. Work starts at `f16705d` on the independent
lab branch; PR #4 remains frozen.

## Experimental boundary

`RON_RESTORE_LIMIT95` requires the post-state observer, which already requires
the broad pre-payload collector. The new opt-in is fixed to the existing
`PathFinderData.limit` word at `0xe85ec0`; its type/offset and 300/95/96 evidence
are in [L68](2026-09-21-path-interventions.md). It is not a general memory editor.
The default observer emits the same version-1 bytes as before.

After a successful pre-payload, the observer requires matching frame/unit,
complete copied-byte count, delegated modes `(300, 1)`, and a fresh read of
those modes. It changes only the limit word to 95, checks readback, then emits
INFO 183. The original function executes with its original arguments and return
path. On return, before any post-state read or file operation, the observer
records the current word and restores 300. It checks both restored modes,
requires the returned word still to have been 95, and emits INFO 184. A refusal
emits INFO 185. It does not alter native EAX, path data, or saved tree records.
If the original never returns, restoration cannot execute; the bounded runner
must terminate that process. This word is process memory, not a persisted setting.

The altered post packet has version 2 and a 32-byte trailer after all retained
path-capacity bytes. Eight little-endian words record provenance version,
address, before, after, saving, at-return limit, restored limit, and success flags:
`(1, 0xe85ec0, 300, 95, 1, 95, 300, 3)`. The embedded original prefix still has
limit 300. Old decoders refuse the new packet version. The updated decoder
requires exact provenance, full extent, matching prefix and return boundary.
Trace validation requires exactly one matching application and restoration,
after pre-payload and surrounding every observed A* entry/return before post-state.
Failure receipts, missing/duplicated/reordered provenance, and intervention
receipts attached to a version-1 control refuse.

## Replay and scope

`native_limit_replay.py` first prepares an unchanged model from that packet.
It then changes exactly the limit word to 95, executes twice using only the
already declared regions, and restores the original model control. Repeats and
control restoration compare full modeled fingerprints. Its native comparison
requires matching intervention metadata and payload hash, and compares EAX,
the entire 344-byte unit and all path-capacity bytes. Only the explicit path
pointer relocation is separated. The ordinary payload replay suppresses its
same-input A* comparison when the trace records an intervention.

This does **not** establish native contents of the retained search trees,
full-state fidelity, or general budget thresholds. The native pre-payload is
unchanged; this capture will not supply an additional unchanged native output.
L67 remains that earlier control. Native disagreement is an experimental result,
not permission to widen masks or change the predicted outcome after the fact.

## Validation and next run

Authored C/Python packets agree for both versions. Negative checks cover every
provenance word, missing trailers, mismatched replay metadata, trace identity
and ordering, and restoring the limit before observer read, identity, extent,
file and trailer failures. Sanitizers pass. Default, payload, post-state and
limit-95 DLLs compile/link in isolated temporary directories; three missing-
dependency configurations refuse compilation. No DLL has been installed live.

The 77 focused Python/C tests pass, as do 256 emitted register/extended-state
adapter cases and their deliberate mutants. The retained L67 native control
still passes `restore_poststate.py --require-agreement`. A fresh offline
preparation using that packet exercised the new comparison workflow against a
**model-derived test boundary**, not a native intervention: it reproduced L68's
165,934 instructions, signed return −1, length/capacity 1/10, exact repeat and
control restoration, with no new region. Its unchanged baseline `last` result
matches L68 in full.

`release_gate.py <owned install> --test-threads 2` exited zero: 332 rondata,
855 sim, 13 fixed-point and three doc tests; all 782 fixture requests present;
clippy, formatting, survey and paperwork checks passed. Peak process-tree
memory was 8,465 MiB. Logs, authored packets, isolated DLLs, dry-run driver and
JSON are retained under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-native-limit-preparation`.
The dry-run JSON explicitly labels its boundary as model-derived. Generated
packets, DLLs and replay reports remain outside git.
Once committed and gated, request one fresh capture slot through Ramon. Use
L67's map-14/seed-12345 scenario, end-frame 1400 and 180-second timeout, adding
only `--tracer-def RON_RESTORE_LIMIT95` to its existing post-state capture flags.
Release the lane as soon as process exit and settings restoration are verified;
validate provenance and replay offline afterward. No slot is assumed from an
older authorization.
