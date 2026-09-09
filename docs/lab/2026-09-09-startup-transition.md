# Startup diagnostics: a WoW64 transition lead

## Established by these runs

The unconditional Media Foundation startup/shutdown calls in `WinMain` both
returned before each of three reproduced failures. The observed timeout is
therefore later than those calls. This does not exclude asynchronous effects
of their work. Two reproduced faults now have a module identity:
`wow64cpu.dll + 0x1139`, inside Wine's 32-to-64-bit syscall transition.
No startup fix or simulation score improvement is claimed.

| Diagnostic cohort | Successful pairs | Successful / launched maps | Failures |
| --- | --- | --- | --- |
| MF witnesses, ordinary Wine logging | 4/4 | 8/8 | None |
| Same witnesses, `WINEDEBUG=+loaddll` | 1/4 | 4/7 | One timeout; two faulting launches |

Both cohorts were fixed at four attempts before running, with no retries and
pair termination on first failure. Endpoint 1400, seed 12345, map 14 then 18,
and a **60-second** process timeout. This differs from the earlier 180-second
reliability cohort. Logging and the one sample can affect scheduling; these
are diagnostic cohorts, not a controlled estimate of logging's effect or a
reliability improvement. All 15 attempted launches restored and byte-verified
five profile files; the game was closed afterward.

All 12 successful maps independently match their earlier baseline on all 19
logged bodies (including closing frame 1401) and all 1401 frame/seed pairs.
This remains observed-projection equality, not complete-state parity.

## Probe and ABI boundary

`unattended_capture.py --startup-probe` builds `RON_STARTUP_PROBE` alongside
`RON_AUTOSTART`. `live_startup_probe.h` wraps only the two import call sites in
`WinMain@00560940`: 0x56096d calls the MFStartup delay-import slot at 0xca9f30;
0x56097a calls MFShutdown's slot at 0xca9f2c. The installed disassembly and
`llvm-readobj --coff-imports` establish those operands. All six bytes of each
indirect call are checked before either is replaced. A mismatch emits the
existing refusal record and cannot pass lifecycle validation.

The replacement calls the original delay-import slot with the original
arguments and preserves its returned value. It neither overwrites the IAT
(which the delay loader manages) nor bypasses either operation. The Start
handler, match execution, and controlled post-match exit are unchanged.

INFO 180 carries site (1 startup / 2 shutdown), phase (0 entry / 1 return),
version, flags, and result. The observed startup arguments are 0x20070 and 0;
both calls returned zero in all 15 launches, including the failures. The
bounded offline `startup_evidence.py` reader retains up to 16 such witnesses
and eight fault contexts; counts disclose overflow. Diagnostic output never
certifies successful acquisition. The ordinary lifecycle reader remains the
acceptance check. Future receipts also record the effective WINEDEBUG setting.

`test_startup_abi.py` compiles the actual header and executes both wrappers
under Unicorn on 256 authored states apiece. It checks original arguments,
return values (including nonzero values), callee-saved registers, caller stack,
and entry/return witnesses. Deliberately changing `ret 8` to `ret 4` fails.
No installed-game code is used in that test. Default and ordinary AUTOSTART
preprocessor tokens match the prior tip exactly when the new opt-in is absent.

## Fault localization and its limit

The loader cohort's pair 2/map 18 loads `wow64cpu.dll` at 0x7bf20000 and faults
at 0x7bf21139 while reading 0x4ecd. Pair 3/map 14 starts with the same fault,
then records two secondary faults at 0x7bf589ce; neither launch reaches a menu
receipt or simulation frame. Pair 1/map 18 times out after the MF return
witnesses, also before menu entry. There is no proof the timeout shares the
fault's cause.

The installed x86-64 Wine 11.0 `wow64cpu.dll` has SHA-256
`cbbf4d054d5488e19d82ea24e7c280c3408ee43b00fce708d1b559acada9d21e`.
Its file-backed instruction at RVA 0x1139 decodes differently by mode:

| Decoder mode | Effective read at the observed load address |
| --- | --- |
| x86-64 | RIP-relative: 0x7bf2600c |
| i386 | Absolute: 0x4ecd |

Both decodes were produced from the installed bytes with `llvm-mc`, not copied
into this repository. The narrow checker `wow64_fault.py MODULE 0x7bf20000
0x7bf21139 0x4ecd` reproduces the effective-address comparison and module hash.
It rejects other instruction forms and explicitly leaves CPU mode unproved.
Three authored PE fixtures test both interpretations, signed displacement,
wrong opcodes, truncated data, and unbacked addresses. Wine 11.0's [upstream syscall transition source](https://github.com/wine-mirror/wine/blob/wine-11.0/dlls/wow64cpu/cpu.c#L160-L190)
identifies the corresponding operation as reading its saved 32-bit code
selector. The local disassembly and reported access address support a
**wrong-mode execution hypothesis**. They do not establish the active segment
selector, Rosetta translation state, or the preceding control-flow error.
Do not patch Wine's transition on that inference alone.

A two-second `/usr/bin/sample` succeeded on pair 2/map 18 at process age nine
seconds, before it subsequently faulted. Its Wine guest-thread stacks contain
repeated opaque transition frames; those repetitions are not evidence of
recursive execution and are not a useful engine flamegraph. The sample is not
a sample of the timed-out launch. The earlier polling sampler was rejected by
automatic approval review; a single bounded invocation was approved and used.

## Reproduction and next falsifier

Run `tools/explore/unattended_capture.py INSTALL FRESH_OUTPUT PROFILE
--end-frame 1400 --timeout 60 --startup-probe`; prefix with `WINEDEBUG=+loaddll`
for module identity. Compare each successful map with
`compare_unattended.py BASELINE_MAP NEW_MAP`, and inspect every failed trace
with `startup_evidence.py TRACE`. Keep failures and unattempted maps distinct.
Do not share the live/profile lane without explicit handoff.

Local evidence: `/tmp/attrition-startup-probe-four-pairs/` and
`/tmp/attrition-startup-loader-four-pairs/`, each with `cohort.json`,
`comparison.json`, and per-map receipts/traces. The sampled process has
`pair-02/map-18/startup-sample.txt` and its receipt in the loader cohort.
`/tmp/attrition-wow64-fault-instruction.json` records the module hash, load
address, RVA, both decoder outputs, and effective-address calculation.
The cohort/compare recipes are `/tmp/run-attrition-startup-probe.py`,
`/tmp/run-attrition-startup-loader.py`, `/tmp/compare-attrition-startup.py`, and
`/tmp/compare-attrition-loader.py`. These artifacts are ephemeral; installed
bytes and game captures stay outside Git.

Next capture should retain CS/SS, flags, and nearby transition state on the
first fault, before secondary exceptions obscure it. Then build a minimal
32-bit Windows bootstrap reproducer to test whether the runtime transition
fails without game assets or renderer setup. If it reproduces, it gives us a
small test for a runtime repair. If not, add coarse engine initialization
witnesses after the now-observed MF pair. Full headless operation and the
canonical command adapter remain downstream of reliable acquisition.
