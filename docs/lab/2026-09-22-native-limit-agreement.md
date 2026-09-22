# Native confirmation of the limit-95 prediction

**The altered native call agrees with the model: return −1, every byte of the
344-byte unit, and all 160 path-capacity bytes. No relocation exception is
needed.** This closes the specific falsifier proposed by L68 and prepared in
[L69](2026-09-22-native-limit-preparation.md). It establishes one useful
counterfactual from a retained call, not a general replacement for the original
runtime. Work starts from `d7a0bdd`; PR #4 and the main scoreboard are unchanged.

## Authorized capture and provenance

Ramon authorized the shared lane on September 22. One run used the previous
map-14/seed-12345 scenario, with only `RON_RESTORE_LIMIT95` added to the collector
configuration. Build plus capture took 29.36 seconds; the game ran 21.60 seconds,
exited zero and reached the requested end-frame 1400. All five backed-up settings
files were restored. The lane was released and Ramon notified immediately,
before offline replay. No further live run was started.

At frame 224, the captured delegated modes were `(300, 1)`. The trace and
version-2 post packet record the one-word change to limit 95, the native A*
call/return, and restoration to 300 before post-state observation. The input
packet remains the unchanged pre-intervention state. Provenance validation
passes; there are no collector failure receipts.

Evidence remains outside git:

- Capture: `/Users/rf-studio/ron-data/lab-captures/2026-09-22-native-limit95/map-14`.
- Payload SHA-256: `1f93c76940beb347b1ff032e0682dcd8aedcd9492d09211071e6a89ba00c4d49`.
- Payload: 818,540,544 bytes in 165 ranges, acquired in 2,037 ms; all 77,156
  anchor bytes validate. This is still a non-atomic acquisition.
- Post packet: 820 bytes, including the explicit 32-byte intervention trailer.
- Retention manifest: 160 capture/staging/report files, including the original
  image binding and restoration receipt.

## Paired replay result

`native_limit_replay.py --require-agreement` prepares the unchanged captured
inputs with the existing bounded malloc/fill/copy/free models. Its control
returns 8 after 197,650 instructions. The control's native A* comparison is
correctly unavailable: this run supplies an altered native output, not another
unchanged native control. L67 remains the earlier unchanged native witness.

The fresh packet needs 42 dependency rounds and 166,128 added captured bytes.
Three explicitly bounded borrowed objects total 672 bytes, 416 beyond the
baseline regions. Their addresses come from this packet's headers, not an
older process. The intervention adds no regions or services.

| observation | native limit 95 | modeled limit 95 |
|---|---:|---:|
| signed outer return | −1 | −1 |
| path length / capacity | 1 / 10 | 1 / 10 |
| unit bytes compared | 344 | 344, zero differences |
| path-capacity bytes compared | 160 | 160, zero differences |
| unit word at +0x148 | 223 | 223 |

The model executes 165,934 instructions, reproducing the prediction recorded
before this capture. Its repeated intervention matches the full modeled
fingerprint; its subsequent unchanged control restores exactly. The five saved
pointer words at unit +0x104 through +0x114 agree as part of the whole-unit
comparison. **The pointed-to trees' post-call contents are not observed.**

The independent `restore_poststate.py --require-agreement` CLI also exits zero
on the intervention replay. A deliberately altered copy flips the last byte
of inactive path slot 9: active-path equality remains true, full-capacity
equality becomes false, and the same CLI exits one with `native path slots
differ`. Neither inactive slots nor arbitrary pointer-shaped values were masked.

## What this buys, and where to stop

This is stronger than reproducing a recorded answer: the lab changed a chosen
input offline, predicted a different outcome, and then observed that outcome
natively at the same call boundary. With L67's unchanged control, there is now
a small measured case for retained-call experiments as a problem-solving tool.
It does not establish a universal 10x speedup, native allocator fidelity,
extended-state import, all mutated state, native limit-96 completion, or general
monotonicity of the budget boundary.

The next useful adoption test is **one actual divergence question selected by
Fable**, with a named field/call and falsifier, rather than more generic heap
or capture machinery. Offer the bounded method and these two witnesses for
review; adoption remains Fable's decision. If that question needs saved-tree
contents, widen that specific post-state observation first. No further capture
is required to finish this slice, and none is booked.

## Validation

The already-gated collector and replay sources are unchanged by this finding.
The live agreement assertion and deliberately failing inactive-slot mutation
are retained, with the capture log and full replay JSON, in
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-native-limit-agreement`.
The post-report release gate exited zero with two test workers: 332 rondata,
855 sim, 13 fixed-point and three doc tests; all 782 fixture requests present;
clippy, formatting, install survey and paperwork checks passed. Peak
process-tree memory was 8,007 MiB. The final validation-note edit was followed
by the fast paperwork guard. No original bytes or generated packet-derived
reports enter git.
